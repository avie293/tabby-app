use super::files::{instance_dir, safe_instance_id};
use super::orchestration::synced_options_path;
use crate::state::InstanceMetadata;
use crate::{ErrorKind, State};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const SAVES_DIRECTORY: &str = "saves";
const LEVEL_DATA_FILE: &str = "level.dat";
const SESSION_LOCK_FILE: &str = "session.lock";
const DETACH_STAGING_DIRECTORY: &str = ".saves.modrinth-detach";

pub(super) fn shared_saves_path(state: &State) -> PathBuf {
    synced_options_path(state).join(SAVES_DIRECTORY)
}

/// Points the instance's `saves` folder at the shared worlds folder.
///
/// Worlds that only exist locally are moved into the shared folder first. A
/// local world whose `level.dat` is byte-identical to the shared world of the
/// same name is an untouched copy and is moved into the backups folder instead.
/// Any other name collision keeps both worlds by renaming the local one.
pub(super) async fn ensure_saves(
    metadata: &InstanceMetadata,
    state: &State,
) -> crate::Result<()> {
    let shared = shared_saves_path(state);
    tokio::fs::create_dir_all(&shared).await?;
    let target = instance_dir(metadata, state).join(SAVES_DIRECTORY);

    match tokio::fs::symlink_metadata(&target).await {
        Ok(entry) if entry.file_type().is_symlink() => {
            if links_to(&target, &shared).await {
                return Ok(());
            }
            remove_link(&target).await?;
        }
        Ok(entry) if entry.is_dir() => {
            merge_into_shared(metadata, &target, &shared, state).await?;
            tokio::fs::remove_dir(&target).await?;
        }
        Ok(_) => {
            return Err(ErrorKind::InputError(
                "The instance's saves path is not a folder".to_string(),
            )
            .into());
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }

    create_dir_link(&shared, &target).await
}

/// Replaces the link with a local copy of every shared world.
///
/// The copy is staged next to the link so a failed copy leaves the instance
/// linked instead of half detached.
pub(super) async fn detach_saves(
    metadata: &InstanceMetadata,
    state: &State,
) -> crate::Result<()> {
    let instance_dir = instance_dir(metadata, state);
    let target = instance_dir.join(SAVES_DIRECTORY);
    let is_link = tokio::fs::symlink_metadata(&target)
        .await
        .is_ok_and(|entry| entry.file_type().is_symlink());
    if !is_link {
        return Ok(());
    }

    let source = tokio::fs::canonicalize(&target).await.ok();
    let staging = instance_dir.join(DETACH_STAGING_DIRECTORY);
    if tokio::fs::symlink_metadata(&staging).await.is_ok() {
        tokio::fs::remove_dir_all(&staging).await?;
    }
    tokio::fs::create_dir_all(&staging).await?;
    if let Some(source) = &source
        && let Err(error) = copy_dir_contents(source, &staging).await
    {
        let _ = tokio::fs::remove_dir_all(&staging).await;
        return Err(error);
    }

    remove_link(&target).await?;
    tokio::fs::rename(&staging, &target).await?;
    Ok(())
}

/// Removes a linked `saves` folder without touching the shared worlds.
pub(crate) async fn unlink_instance_saves(
    instance_dir: &Path,
) -> crate::Result<()> {
    let target = instance_dir.join(SAVES_DIRECTORY);
    let is_link = tokio::fs::symlink_metadata(&target)
        .await
        .is_ok_and(|entry| entry.file_type().is_symlink());
    if is_link {
        remove_link(&target).await?;
    }
    Ok(())
}

/// Removes every linked `saves` folder below `instances_dir`.
///
/// Used before the app directory is moved, since the links point at absolute
/// paths. Participating instances are linked again on the next reconcile.
pub(crate) async fn unlink_all_instance_saves(
    instances_dir: &Path,
) -> crate::Result<()> {
    let mut entries = match tokio::fs::read_dir(instances_dir).await {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(());
        }
        Err(error) => return Err(error.into()),
    };
    while let Some(entry) = entries.next_entry().await? {
        if entry.file_type().await?.is_dir() {
            unlink_instance_saves(&entry.path()).await?;
        }
    }
    Ok(())
}

async fn merge_into_shared(
    metadata: &InstanceMetadata,
    local: &Path,
    shared: &Path,
    state: &State,
) -> crate::Result<()> {
    let mut backup_dir = None;
    let mut entries = tokio::fs::read_dir(local).await?;
    while let Some(entry) = entries.next_entry().await? {
        let name = entry.file_name();
        let source = entry.path();
        let destination = shared.join(&name);
        if tokio::fs::symlink_metadata(&destination).await.is_err() {
            move_path(&source, &destination).await?;
            continue;
        }

        if same_level_data(&source, &destination).await {
            let backup = match backup_dir.take() {
                Some(path) => path,
                None => saves_backup_dir(&metadata.instance.id, state).await?,
            };
            move_path(&source, &backup.join(&name)).await?;
            backup_dir = Some(backup);
            continue;
        }

        let destination = available_name(
            shared,
            &name.to_string_lossy(),
            &metadata.instance.name,
        )
        .await;
        move_path(&source, &destination).await?;
    }
    Ok(())
}

async fn same_level_data(a: &Path, b: &Path) -> bool {
    let (Ok(a), Ok(b)) = (
        tokio::fs::read(a.join(LEVEL_DATA_FILE)).await,
        tokio::fs::read(b.join(LEVEL_DATA_FILE)).await,
    ) else {
        return false;
    };
    a == b
}

async fn saves_backup_dir(
    instance_id: &str,
    state: &State,
) -> crate::Result<PathBuf> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let directory = synced_options_path(state)
        .join("backups")
        .join(safe_instance_id(instance_id))
        .join(timestamp.to_string())
        .join(SAVES_DIRECTORY);
    tokio::fs::create_dir_all(&directory).await?;
    Ok(directory)
}

async fn available_name(
    directory: &Path,
    name: &str,
    instance_name: &str,
) -> PathBuf {
    let instance_name = instance_name
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || matches!(c, ' ' | '-' | '_' | '.') {
                c
            } else {
                '_'
            }
        })
        .collect::<String>();
    let instance_name = instance_name.trim();
    let base = if instance_name.is_empty() {
        format!("{name} (synced)")
    } else {
        format!("{name} ({instance_name})")
    };
    let mut candidate = directory.join(&base);
    let mut counter = 2;
    while tokio::fs::symlink_metadata(&candidate).await.is_ok() {
        candidate = directory.join(format!("{base} {counter}"));
        counter += 1;
    }
    candidate
}

async fn move_path(source: &Path, destination: &Path) -> crate::Result<()> {
    if tokio::fs::rename(source, destination).await.is_ok() {
        return Ok(());
    }
    if tokio::fs::symlink_metadata(source).await?.is_dir() {
        tokio::fs::create_dir_all(destination).await?;
        copy_dir_contents(source, destination).await?;
        tokio::fs::remove_dir_all(source).await?;
    } else {
        tokio::fs::copy(source, destination).await?;
        tokio::fs::remove_file(source).await?;
    }
    Ok(())
}

/// Copies a directory tree, skipping Minecraft's world lock files.
async fn copy_dir_contents(
    source: &Path,
    destination: &Path,
) -> crate::Result<()> {
    let mut pending = vec![(source.to_path_buf(), destination.to_path_buf())];
    while let Some((source, destination)) = pending.pop() {
        let mut entries = tokio::fs::read_dir(&source).await?;
        while let Some(entry) = entries.next_entry().await? {
            let entry_destination = destination.join(entry.file_name());
            let file_type = entry.file_type().await?;
            if file_type.is_dir() {
                tokio::fs::create_dir_all(&entry_destination).await?;
                pending.push((entry.path(), entry_destination));
            } else if file_type.is_file()
                && entry.file_name() != SESSION_LOCK_FILE
            {
                tokio::fs::copy(entry.path(), &entry_destination).await?;
            }
        }
    }
    Ok(())
}

async fn links_to(link: &Path, expected: &Path) -> bool {
    match (
        tokio::fs::canonicalize(link).await,
        tokio::fs::canonicalize(expected).await,
    ) {
        (Ok(link), Ok(expected)) => link == expected,
        _ => false,
    }
}

async fn remove_link(path: &Path) -> crate::Result<()> {
    #[cfg(windows)]
    {
        if tokio::fs::remove_dir(path).await.is_ok() {
            return Ok(());
        }
    }
    tokio::fs::remove_file(path).await?;
    Ok(())
}

#[cfg(unix)]
async fn create_dir_link(source: &Path, target: &Path) -> crate::Result<()> {
    tokio::fs::symlink(source, target).await?;
    Ok(())
}

/// Creates a directory junction, which unlike a directory symlink does not
/// require administrator rights or developer mode.
#[cfg(windows)]
async fn create_dir_link(source: &Path, target: &Path) -> crate::Result<()> {
    use std::os::windows::process::CommandExt;

    const CREATE_NO_WINDOW: u32 = 0x0800_0000;

    let source = dunce::simplified(source).to_path_buf();
    let target = dunce::simplified(target).to_path_buf();
    let output = tokio::task::spawn_blocking(move || {
        std::process::Command::new("cmd")
            .raw_arg(format!(
                "/C mklink /J \"{}\" \"{}\"",
                target.display(),
                source.display()
            ))
            .creation_flags(CREATE_NO_WINDOW)
            .output()
    })
    .await??;
    if !output.status.success() {
        return Err(ErrorKind::FSError(format!(
            "Could not link the synced worlds folder: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ))
        .into());
    }
    Ok(())
}
