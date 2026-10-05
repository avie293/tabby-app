//! CurseForge modpack support: installing CurseForge modpack zips and
//! describing instances in CurseForge's manifest format for export.
//!
//! A CurseForge modpack zip only references its mods by project and file id,
//! so installing one is done by converting it into an equivalent `.mrpack`
//! whose downloads point at CurseForge (or Modrinth, for files whose authors
//! do not allow third-party downloads) and installing that.

use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::json;

use super::install_from::{
    CreatePackLocation, PackDependency, PackFile, PackFileHash, PackFormat,
};
use crate::util::fetch::sha1_file_async;
use crate::util::io;
use crate::{ErrorKind, State};

const API_URL: &str = "https://api.curseforge.com/v1";
const MINECRAFT_GAME_ID: u32 = 432;
const MANIFEST_FILE: &str = "manifest.json";
const MRPACK_INDEX_FILE: &str = "modrinth.index.json";

/// The CurseForge API key baked in at build time, if one was configured.
pub(crate) fn api_key() -> Option<&'static str> {
    option_env!("CURSEFORGE_API_KEY").filter(|key| !key.is_empty())
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Manifest {
    pub minecraft: ManifestMinecraft,
    #[serde(default = "default_manifest_type")]
    pub manifest_type: String,
    #[serde(default = "default_manifest_version")]
    pub manifest_version: u32,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub files: Vec<ManifestFile>,
    #[serde(default = "default_overrides")]
    pub overrides: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ManifestMinecraft {
    pub version: String,
    #[serde(default)]
    pub mod_loaders: Vec<ManifestModLoader>,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct ManifestModLoader {
    pub id: String,
    #[serde(default)]
    pub primary: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub(crate) struct ManifestFile {
    #[serde(rename = "projectID")]
    pub project_id: u32,
    #[serde(rename = "fileID")]
    pub file_id: u32,
    #[serde(default = "default_required")]
    pub required: bool,
}

fn default_manifest_type() -> String {
    "minecraftModpack".to_string()
}

fn default_manifest_version() -> u32 {
    1
}

fn default_overrides() -> String {
    "overrides".to_string()
}

fn default_required() -> bool {
    true
}

#[derive(Deserialize)]
struct ApiList<T> {
    data: Vec<T>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ApiFile {
    mod_id: u32,
    file_name: String,
    download_url: Option<String>,
    file_length: u64,
    #[serde(default)]
    hashes: Vec<ApiFileHash>,
}

#[derive(Deserialize)]
struct ApiFileHash {
    value: String,
    algo: u32,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ApiMod {
    id: u32,
    name: String,
    class_id: Option<u32>,
    links: Option<ApiModLinks>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ApiModLinks {
    website_url: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct FingerprintMatches {
    exact_matches: Vec<FingerprintMatch>,
}

#[derive(Deserialize)]
struct FingerprintMatch {
    id: u32,
    file: FingerprintFile,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct FingerprintFile {
    id: u32,
    file_fingerprint: u32,
}

#[derive(Deserialize)]
struct ModrinthVersion {
    files: Vec<ModrinthVersionFile>,
}

#[derive(Deserialize)]
struct ModrinthVersionFile {
    url: String,
    size: u64,
    hashes: HashMap<String, String>,
}

/// CurseForge's file fingerprint: MurmurHash2 with seed 1 over the file's
/// bytes with all whitespace removed.
pub(crate) fn fingerprint(bytes: &[u8]) -> u32 {
    let normalized = bytes
        .iter()
        .copied()
        .filter(|byte| !matches!(byte, 9 | 10 | 13 | 32))
        .collect::<Vec<_>>();
    murmur2::murmur2(&normalized, 1)
}

fn client() -> crate::Result<reqwest::Client> {
    Ok(reqwest::Client::builder()
        .user_agent(crate::launcher_user_agent())
        .build()?)
}

fn require_api_key() -> crate::Result<&'static str> {
    api_key().ok_or_else(|| {
        ErrorKind::InputError(
            "This build has no CurseForge API key, so CurseForge modpacks cannot be installed."
                .to_string(),
        )
        .into()
    })
}

async fn post_api<T: for<'de> Deserialize<'de>>(
    path: &str,
    body: serde_json::Value,
) -> crate::Result<T> {
    let response = client()?
        .post(format!("{API_URL}{path}"))
        .header("x-api-key", require_api_key()?)
        .json(&body)
        .send()
        .await?
        .error_for_status()?;
    Ok(response.json().await?)
}

/// Looks up which local files are known CurseForge files. Returns the
/// CurseForge project and file id for each matching path.
pub(crate) async fn match_files_by_fingerprint(
    fingerprints: &HashMap<String, u32>,
) -> crate::Result<HashMap<String, ManifestFile>> {
    if fingerprints.is_empty() || api_key().is_none() {
        return Ok(HashMap::new());
    }
    let unique = fingerprints.values().copied().collect::<HashSet<_>>();
    let matches: FingerprintMatches =
        post_api::<ApiObject<FingerprintMatches>>(
            &format!("/fingerprints/{MINECRAFT_GAME_ID}"),
            json!({ "fingerprints": unique }),
        )
        .await?
        .data;
    let by_fingerprint = matches
        .exact_matches
        .into_iter()
        .map(|found| {
            (
                found.file.file_fingerprint,
                ManifestFile {
                    project_id: found.id,
                    file_id: found.file.id,
                    required: true,
                },
            )
        })
        .collect::<HashMap<_, _>>();
    Ok(fingerprints
        .iter()
        .filter_map(|(path, fingerprint)| {
            by_fingerprint
                .get(fingerprint)
                .map(|file| (path.clone(), *file))
        })
        .collect())
}

#[derive(Deserialize)]
struct ApiObject<T> {
    data: T,
}

pub(crate) fn manifest_mod_loader(
    loader: crate::state::ModLoader,
    loader_version: Option<&str>,
) -> Option<ManifestModLoader> {
    use crate::state::ModLoader;
    let prefix = match loader {
        ModLoader::Forge => "forge",
        ModLoader::NeoForge => "neoforge",
        ModLoader::Fabric => "fabric",
        ModLoader::Quilt => "quilt",
        ModLoader::Vanilla => return None,
    };
    Some(ManifestModLoader {
        id: format!("{prefix}-{}", loader_version?),
        primary: true,
    })
}

/// Converts CurseForge modpack zips into `.mrpack` files so the regular
/// modpack installer can handle them. Other locations are returned as is.
pub async fn normalize_pack_location(
    location: CreatePackLocation,
) -> crate::Result<CreatePackLocation> {
    let CreatePackLocation::FromFile { path } = &location else {
        return Ok(location);
    };
    if !path.is_file() || !is_curseforge_zip(path).await? {
        return Ok(location);
    }
    if !CURSEFORGE_INSTALL_ENABLED {
        return Err(ErrorKind::InputError(
            "Installing CurseForge modpacks will be supported in a later version."
                .to_string(),
        )
        .into());
    }
    let converted = convert_to_mrpack(path).await?;
    Ok(CreatePackLocation::FromFile { path: converted })
}

/// Installing CurseForge modpacks is unfinished and turned off for now.
const CURSEFORGE_INSTALL_ENABLED: bool = false;

async fn is_curseforge_zip(path: &Path) -> crate::Result<bool> {
    let path = path.to_path_buf();
    tokio::task::spawn_blocking(move || {
        let Ok(archive) = File::open(&path).map(zip::ZipArchive::new) else {
            return Ok(false);
        };
        let Ok(archive) = archive else {
            return Ok(false);
        };
        let names = archive.file_names().collect::<HashSet<_>>();
        Ok(names.contains(MANIFEST_FILE) && !names.contains(MRPACK_INDEX_FILE))
    })
    .await?
}

fn read_manifest(path: &Path) -> crate::Result<Manifest> {
    let mut archive = zip::ZipArchive::new(File::open(path)?)
        .map_err(std::io::Error::from)?;
    let manifest = archive
        .by_name(MANIFEST_FILE)
        .map_err(std::io::Error::from)?;
    let manifest: Manifest = serde_json::from_reader(manifest)?;
    if manifest.manifest_type != "minecraftModpack" {
        return Err(ErrorKind::InputError(
            "This zip is not a CurseForge modpack.".to_string(),
        )
        .into());
    }
    Ok(manifest)
}

/// Picks the folder for a CurseForge file from its project's class.
fn folder_for_class(class_id: Option<u32>) -> &'static str {
    match class_id {
        Some(12) => "resourcepacks",
        Some(6552) => "shaderpacks",
        Some(6945) => "datapacks",
        _ => "mods",
    }
}

fn pack_dependencies(
    manifest: &Manifest,
) -> crate::Result<HashMap<PackDependency, String>> {
    let mut dependencies = HashMap::from([(
        PackDependency::Minecraft,
        manifest.minecraft.version.clone(),
    )]);
    let loader = manifest
        .minecraft
        .mod_loaders
        .iter()
        .find(|loader| loader.primary)
        .or_else(|| manifest.minecraft.mod_loaders.first());
    if let Some(loader) = loader {
        let (name, version) = loader.id.split_once('-').ok_or_else(|| {
            ErrorKind::InputError(format!(
                "Unknown CurseForge mod loader {}",
                loader.id
            ))
        })?;
        let dependency = match name {
            "forge" => PackDependency::Forge,
            "neoforge" => PackDependency::NeoForge,
            "fabric" => PackDependency::FabricLoader,
            "quilt" => PackDependency::QuiltLoader,
            _ => {
                return Err(ErrorKind::InputError(format!(
                    "Unsupported CurseForge mod loader {name}"
                ))
                .into());
            }
        };
        let version = version
            .strip_prefix(&format!("{}-", manifest.minecraft.version))
            .unwrap_or(version);
        dependencies.insert(dependency, version.to_string());
    }
    Ok(dependencies)
}

async fn convert_to_mrpack(path: &Path) -> crate::Result<PathBuf> {
    let state = State::get().await?;
    let (_, archive_hash) = sha1_file_async(path).await?;
    let stem = path
        .file_stem()
        .map(|stem| stem.to_string_lossy().to_string())
        .unwrap_or_else(|| "CurseForge modpack".to_string());
    let output_dir = state
        .directories
        .caches_dir()
        .join("curseforge-packs")
        .join(&archive_hash);
    let output = output_dir.join(format!("{stem}.mrpack"));
    if output.is_file() {
        return Ok(output);
    }

    let manifest = {
        let path = path.to_path_buf();
        tokio::task::spawn_blocking(move || read_manifest(&path)).await??
    };
    let dependencies = pack_dependencies(&manifest)?;

    let file_ids = manifest
        .files
        .iter()
        .map(|file| file.file_id)
        .collect::<Vec<_>>();
    let project_ids = manifest
        .files
        .iter()
        .map(|file| file.project_id)
        .collect::<HashSet<_>>();
    let (files, projects) = if file_ids.is_empty() {
        (Vec::new(), Vec::new())
    } else {
        let files: ApiList<ApiFile> =
            post_api("/mods/files", json!({ "fileIds": file_ids })).await?;
        let projects: ApiList<ApiMod> =
            post_api("/mods", json!({ "modIds": project_ids })).await?;
        (files.data, projects.data)
    };
    let projects = projects
        .into_iter()
        .map(|project| (project.id, project))
        .collect::<HashMap<_, _>>();

    let blocked_sha1 = files
        .iter()
        .filter(|file| file.download_url.is_none())
        .filter_map(sha1_of)
        .collect::<Vec<_>>();
    let modrinth_fallbacks = find_on_modrinth(&blocked_sha1).await;

    let mut pack_files = Vec::with_capacity(files.len());
    let mut missing = Vec::new();
    for file in &files {
        let project = projects.get(&file.mod_id);
        let folder = folder_for_class(project.and_then(|p| p.class_id));
        let sha1 = sha1_of(file);
        let (download, sha512, size) = match &file.download_url {
            Some(url) => (url.clone(), None, file.file_length),
            None => match sha1.as_ref().and_then(|s| modrinth_fallbacks.get(s))
            {
                Some(found) => (
                    found.url.clone(),
                    found.hashes.get("sha512").cloned(),
                    found.size,
                ),
                None => {
                    missing.push(match project {
                        Some(project) => match project
                            .links
                            .as_ref()
                            .and_then(|links| links.website_url.as_ref())
                        {
                            Some(url) => format!("{} ({url})", project.name),
                            None => project.name.clone(),
                        },
                        None => file.file_name.clone(),
                    });
                    continue;
                }
            },
        };
        let mut hashes = HashMap::new();
        if let Some(sha1) = sha1 {
            hashes.insert(PackFileHash::Sha1, sha1);
        }
        if let Some(sha512) = sha512 {
            hashes.insert(PackFileHash::Sha512, sha512);
        }
        pack_files.push(PackFile {
            path: format!("{folder}/{}", file.file_name).try_into()?,
            hashes,
            env: None,
            downloads: vec![download],
            file_size: u32::try_from(size).unwrap_or(u32::MAX),
        });
    }

    let index = PackFormat {
        game: "minecraft".to_string(),
        format_version: 1,
        version_id: if manifest.version.is_empty() {
            "1.0.0".to_string()
        } else {
            manifest.version.clone()
        },
        name: if manifest.name.is_empty() {
            stem.clone()
        } else {
            manifest.name.clone()
        },
        summary: None,
        files: pack_files,
        dependencies,
    };
    let index = serde_json::to_vec_pretty(&index)?;

    io::create_dir_all(&output_dir).await?;
    let source = path.to_path_buf();
    let overrides = format!("{}/", manifest.overrides.trim_end_matches('/'));
    let temporary = output_dir.join(format!("{stem}.mrpack.tmp"));
    {
        let temporary = temporary.clone();
        tokio::task::spawn_blocking(move || {
            write_mrpack(&source, &temporary, &overrides, &index)
        })
        .await??;
    }
    tokio::fs::rename(&temporary, &output).await?;

    if !missing.is_empty() {
        let _ = crate::event::emit::emit_warning(&format!(
            "{} mod(s) from this CurseForge modpack can only be downloaded manually and were skipped: {}",
            missing.len(),
            missing.join(", ")
        ))
        .await;
    }
    Ok(output)
}

fn sha1_of(file: &ApiFile) -> Option<String> {
    file.hashes
        .iter()
        .find(|hash| hash.algo == 1)
        .map(|hash| hash.value.to_lowercase())
}

/// Finds files whose CurseForge authors disabled third-party downloads on
/// Modrinth, by their SHA-1 hash.
async fn find_on_modrinth(
    sha1_hashes: &[String],
) -> HashMap<String, ModrinthVersionFile> {
    if sha1_hashes.is_empty() {
        return HashMap::new();
    }
    let response = async {
        let response = client()?
            .post(concat!(env!("MODRINTH_API_URL"), "version_files"))
            .json(&json!({ "hashes": sha1_hashes, "algorithm": "sha1" }))
            .send()
            .await?
            .error_for_status()?;
        Ok::<_, crate::Error>(
            response.json::<HashMap<String, ModrinthVersion>>().await?,
        )
    }
    .await;
    let Ok(versions) = response else {
        return HashMap::new();
    };
    versions
        .into_iter()
        .filter_map(|(sha1, version)| {
            let file = version.files.into_iter().find(|file| {
                file.hashes.get("sha1").is_some_and(|hash| *hash == sha1)
            })?;
            Some((sha1, file))
        })
        .collect()
}

fn write_mrpack(
    source: &Path,
    destination: &Path,
    overrides: &str,
    index: &[u8],
) -> crate::Result<()> {
    use std::io::Write;

    let mut archive = zip::ZipArchive::new(File::open(source)?)
        .map_err(std::io::Error::from)?;
    let mut writer = zip::ZipWriter::new(File::create(destination)?);
    for entry_index in 0..archive.len() {
        let entry = archive
            .by_index_raw(entry_index)
            .map_err(std::io::Error::from)?;
        let name = entry.name().to_string();
        let Some(relative) = name.strip_prefix(overrides) else {
            continue;
        };
        if relative.is_empty() || name.ends_with('/') {
            continue;
        }
        writer
            .raw_copy_file_rename(entry, format!("overrides/{relative}"))
            .map_err(std::io::Error::from)?;
    }
    writer
        .start_file(
            MRPACK_INDEX_FILE,
            zip::write::SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Deflated),
        )
        .map_err(std::io::Error::from)?;
    writer.write_all(index)?;
    writer.finish().map_err(std::io::Error::from)?;
    Ok(())
}
