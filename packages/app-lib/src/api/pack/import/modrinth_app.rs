//! Imports instances, their files and playtime from an installed official
//! Modrinth App.
//!
//! The Modrinth App keeps its instances in the same SQLite schema as this app.
//! Its database is copied to a snapshot before reading, so a running Modrinth
//! App is never locked or modified.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use tokio::sync::Mutex;

use crate::install::{
    InstallJobSnapshot, InstallPhaseDetails, InstallProgressReporter,
};
use crate::prelude::ModLoader;
use crate::state::{
    AppliedContentSetPatch, EditInstance, InstanceInstallStage,
};
use crate::util::io;
use crate::{ErrorKind, State};

use super::{
    ImportLauncherType, copy_dotminecraft_filtered_with_reporter, recache_icon,
};

const MODRINTH_APP_IDENTIFIER: &str = "ModrinthApp";
const MANIFEST_FILE: &str = "tabbyapp-import.json";
const HISTORY_FILE: &str = "modrinth-app-import.json";
const CONTENT_FOLDERS: &[&str] =
    &["mods", "resourcepacks", "shaderpacks", "datapacks"];
const SKIPPED_FOLDERS: &[&str] = &["logs", "crash-reports", ".mixin.out"];

static HISTORY_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ModrinthAppInstance {
    pub id: String,
    pub name: String,
    pub folder: String,
    pub game_version: Option<String>,
    pub loader: String,
    pub loader_version: Option<String>,
    pub playtime_seconds: u64,
    pub last_played: Option<i64>,
    pub icon_path: Option<String>,
}

/// Which parts of an instance to import. Anything that is not content,
/// worlds or screenshots (configs, options.txt, servers.dat, ...) counts as
/// `settings`.
#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct ModrinthAppImportOptions {
    pub content: bool,
    pub worlds: bool,
    pub screenshots: bool,
    pub settings: bool,
    pub playtime: bool,
}

impl ModrinthAppImportOptions {
    fn includes(&self, relative_path: &str) -> bool {
        let folder = relative_path.split('/').next().unwrap_or_default();
        if SKIPPED_FOLDERS.contains(&folder) {
            return false;
        }
        if CONTENT_FOLDERS.contains(&folder) {
            return self.content;
        }
        match folder {
            "saves" => self.worlds,
            "screenshots" => self.screenshots,
            _ => self.settings,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ImportedInstanceRecord {
    pub instance_id: String,
    pub imported_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PlaytimeTransferRecord {
    pub instance_id: String,
    pub seconds: u64,
    pub transferred_at: DateTime<Utc>,
}

/// What has already been imported, keyed by Modrinth App instance id, so the
/// UI can warn before importing twice or counting playtime twice.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ModrinthAppImportHistory {
    #[serde(default)]
    pub imports: HashMap<String, ImportedInstanceRecord>,
    #[serde(default)]
    pub playtime_transfers: HashMap<String, PlaytimeTransferRecord>,
}

#[derive(Serialize, Deserialize)]
struct ImportManifest {
    source_dir: PathBuf,
    instance: ModrinthAppInstance,
    options: ModrinthAppImportOptions,
}

pub fn modrinth_app_settings_dir() -> Option<PathBuf> {
    let path = dirs::data_dir()?.join(MODRINTH_APP_IDENTIFIER);
    path.join("app.db").exists().then_some(path)
}

pub async fn list_instances() -> crate::Result<Vec<ModrinthAppInstance>> {
    let state = State::get().await?;
    let (pool, profiles_dir) = open_snapshot(&state).await?;
    let instances = read_instances(&pool, &profiles_dir).await;
    pool.close().await;
    Ok(instances?
        .into_iter()
        .map(|(instance, _)| instance)
        .collect())
}

/// Starts one import job per selected Modrinth App instance.
pub async fn import_instances(
    instance_ids: &[String],
    options: ModrinthAppImportOptions,
) -> crate::Result<Vec<InstallJobSnapshot>> {
    let state = State::get().await?;
    let (pool, profiles_dir) = open_snapshot(&state).await?;
    let instances = read_instances(&pool, &profiles_dir).await;
    pool.close().await;

    let mut jobs = Vec::new();
    for (instance, source_dir) in instances? {
        if !instance_ids.contains(&instance.id) {
            continue;
        }
        let staging_root = import_cache_dir(&state)
            .join("jobs")
            .join(uuid::Uuid::new_v4().to_string());
        let folder = instance.folder.clone();
        io::create_dir_all(staging_root.join(&folder)).await?;
        let manifest = ImportManifest {
            source_dir,
            instance,
            options,
        };
        io::write(
            staging_root.join(&folder).join(MANIFEST_FILE),
            serde_json::to_vec(&manifest)?,
        )
        .await?;
        jobs.push(
            crate::install::import_instance(
                ImportLauncherType::ModrinthApp,
                staging_root,
                folder,
            )
            .await?,
        );
    }
    Ok(jobs)
}

pub(super) async fn is_staged_import(instance_path: &Path) -> bool {
    instance_path.join(MANIFEST_FILE).exists()
}

pub(super) async fn import_modrinth_app(
    staging_root: PathBuf,
    instance_folder: String,
    instance_id: &str,
    reporter: InstallProgressReporter,
    details: InstallPhaseDetails,
) -> crate::Result<()> {
    let manifest: ImportManifest = serde_json::from_slice(
        &io::read(staging_root.join(&instance_folder).join(MANIFEST_FILE))
            .await?,
    )?;
    let ImportManifest {
        source_dir,
        instance,
        options,
    } = manifest;

    let game_version = instance.game_version.clone().ok_or_else(|| {
        ErrorKind::InputError(format!(
            "{} has no Minecraft version and cannot be imported.",
            instance.name
        ))
    })?;
    let loader = ModLoader::from_string(&instance.loader);
    let loader_version = if loader == ModLoader::Vanilla {
        None
    } else {
        crate::launcher::get_loader_version_from_profile(
            &game_version,
            loader,
            instance.loader_version.as_deref(),
        )
        .await?
        .map(|version| version.id)
    };
    let icon = match &instance.icon_path {
        Some(path) => recache_icon(PathBuf::from(path)).await.ok().flatten(),
        None => None,
    };

    crate::api::instance::edit(
        instance_id,
        EditInstance {
            install_stage: Some(InstanceInstallStage::PackInstalling),
            name: Some(instance.name.clone()),
            icon_path: Some(
                icon.map(|path| path.to_string_lossy().to_string()),
            ),
            content_set_patch: Some(AppliedContentSetPatch {
                source_kind: None,
                game_version: Some(game_version),
                protocol_version: Some(None),
                loader: Some(loader),
                loader_version: Some(loader_version),
            }),
            submitted_time_played: options
                .playtime
                .then_some(instance.playtime_seconds),
            last_played: options.playtime.then(|| {
                instance
                    .last_played
                    .and_then(|seconds| DateTime::from_timestamp(seconds, 0))
            }),
            ..EditInstance::default()
        },
    )
    .await?;

    let state = State::get().await?;
    copy_dotminecraft_filtered_with_reporter(
        instance_id,
        source_dir,
        &state.io_semaphore,
        reporter.clone(),
        details,
        |relative_path| options.includes(relative_path),
    )
    .await?;

    crate::launcher::install_minecraft_for_instance_id_with_reporter(
        instance_id,
        false,
        Some(reporter),
    )
    .await?;

    update_history(&state, |history| {
        history.imports.insert(
            instance.id.clone(),
            ImportedInstanceRecord {
                instance_id: instance_id.to_string(),
                imported_at: Utc::now(),
            },
        );
        if options.playtime {
            history.playtime_transfers.insert(
                instance.id.clone(),
                PlaytimeTransferRecord {
                    instance_id: instance_id.to_string(),
                    seconds: instance.playtime_seconds,
                    transferred_at: Utc::now(),
                },
            );
        }
    })
    .await?;

    let _ = io::remove_dir_all(&staging_root).await;
    Ok(())
}

/// Adds a Modrinth App instance's playtime to an existing instance.
///
/// The playtime is added as already submitted playtime so it is not reported
/// to Modrinth's analytics a second time.
pub async fn transfer_playtime(
    source_instance_id: &str,
    target_instance_id: &str,
) -> crate::Result<u64> {
    let state = State::get().await?;
    let (pool, profiles_dir) = open_snapshot(&state).await?;
    let instances = read_instances(&pool, &profiles_dir).await;
    pool.close().await;
    let source = instances?
        .into_iter()
        .map(|(instance, _)| instance)
        .find(|instance| instance.id == source_instance_id)
        .ok_or_else(|| {
            ErrorKind::InputError(
                "That Modrinth App instance no longer exists.".to_string(),
            )
        })?;

    let target = crate::state::get_instance(target_instance_id, &state.pool)
        .await?
        .ok_or_else(|| ErrorKind::InputError("Unknown instance".to_string()))?;
    let source_last_played = source
        .last_played
        .and_then(|seconds| DateTime::from_timestamp(seconds, 0));
    let last_played = match (target.instance.last_played, source_last_played) {
        (Some(target), Some(source)) => Some(target.max(source)),
        (target, source) => target.or(source),
    };

    crate::api::instance::edit(
        target_instance_id,
        EditInstance {
            submitted_time_played: Some(
                target.instance.submitted_time_played + source.playtime_seconds,
            ),
            last_played: Some(last_played),
            ..EditInstance::default()
        },
    )
    .await?;

    update_history(&state, |history| {
        history.playtime_transfers.insert(
            source.id.clone(),
            PlaytimeTransferRecord {
                instance_id: target_instance_id.to_string(),
                seconds: source.playtime_seconds,
                transferred_at: Utc::now(),
            },
        );
    })
    .await?;

    Ok(source.playtime_seconds)
}

pub async fn get_history() -> crate::Result<ModrinthAppImportHistory> {
    let state = State::get().await?;
    let _guard = HISTORY_LOCK.lock().await;
    read_history(&state).await
}

fn import_cache_dir(state: &State) -> PathBuf {
    state.directories.caches_dir().join("modrinth-app-import")
}

/// Copies the Modrinth App database (including its write-ahead log) and opens
/// the copy. Returns the pool and the Modrinth App's instances folder.
async fn open_snapshot(state: &State) -> crate::Result<(SqlitePool, PathBuf)> {
    let settings_dir = modrinth_app_settings_dir().ok_or_else(|| {
        ErrorKind::InputError(
            "The Modrinth App was not found on this computer.".to_string(),
        )
    })?;
    let snapshot_dir = import_cache_dir(state).join("database");
    io::create_dir_all(&snapshot_dir).await?;
    for suffix in ["", "-wal", "-shm"] {
        let source = settings_dir.join(format!("app.db{suffix}"));
        let target = snapshot_dir.join(format!("app.db{suffix}"));
        if source.exists() {
            tokio::fs::copy(&source, &target).await?;
        } else if target.exists() {
            io::remove_file(&target).await?;
        }
    }

    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            SqliteConnectOptions::new().filename(snapshot_dir.join("app.db")),
        )
        .await?;
    let custom_dir: Option<String> =
        sqlx::query_scalar("SELECT custom_dir FROM settings LIMIT 1")
            .fetch_optional(&pool)
            .await?
            .flatten();
    let root = custom_dir
        .filter(|path| !path.is_empty())
        .map(PathBuf::from)
        .unwrap_or(settings_dir);
    Ok((pool, root.join("profiles")))
}

async fn read_instances(
    pool: &SqlitePool,
    profiles_dir: &Path,
) -> crate::Result<Vec<(ModrinthAppInstance, PathBuf)>> {
    let rows = sqlx::query(
        "
		SELECT
			instances.id,
			instances.name,
			instances.path,
			instances.icon_path,
			instances.last_played,
			instances.submitted_time_played,
			instances.recent_time_played,
			content_sets.game_version,
			content_sets.loader,
			content_sets.loader_version
		FROM instances
		LEFT JOIN instance_content_sets content_sets
			ON content_sets.id = instances.applied_content_set_id
		WHERE instances.install_stage = 'installed'
		ORDER BY instances.last_played DESC
		",
    )
    .fetch_all(pool)
    .await?;

    let mut instances = Vec::with_capacity(rows.len());
    for row in rows {
        let folder: String = row.try_get("path")?;
        let source_dir = profiles_dir.join(&folder);
        if !source_dir.is_dir() {
            continue;
        }
        let submitted: i64 = row.try_get("submitted_time_played")?;
        let recent: i64 = row.try_get("recent_time_played")?;
        instances.push((
            ModrinthAppInstance {
                id: row.try_get("id")?,
                name: row.try_get("name")?,
                folder,
                game_version: row.try_get("game_version")?,
                loader: row
                    .try_get::<Option<String>, _>("loader")?
                    .unwrap_or_else(|| "vanilla".to_string()),
                loader_version: row.try_get("loader_version")?,
                playtime_seconds: (submitted.max(0) + recent.max(0)) as u64,
                last_played: row.try_get("last_played")?,
                icon_path: row.try_get("icon_path")?,
            },
            source_dir,
        ));
    }
    Ok(instances)
}

async fn read_history(
    state: &State,
) -> crate::Result<ModrinthAppImportHistory> {
    let path = state.directories.settings_dir.join(HISTORY_FILE);
    if !path.exists() {
        return Ok(ModrinthAppImportHistory::default());
    }
    Ok(serde_json::from_slice(&io::read(&path).await?).unwrap_or_default())
}

async fn update_history(
    state: &State,
    update: impl FnOnce(&mut ModrinthAppImportHistory),
) -> crate::Result<()> {
    let _guard = HISTORY_LOCK.lock().await;
    let mut history = read_history(state).await?;
    update(&mut history);
    io::write(
        state.directories.settings_dir.join(HISTORY_FILE),
        serde_json::to_vec_pretty(&history)?,
    )
    .await?;
    Ok(())
}
