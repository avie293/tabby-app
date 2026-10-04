use super::files::{
    CheckpointStatus, begin_checkpoint, checkpoint, ensure_link,
    finish_checkpoint, instance_dir, sha1_bytes, sha1_file,
};
use super::orchestration::{
    backup_bytes, option_effective, synced_options_path,
};
use crate::State;
use crate::state::{InstanceMetadata, SyncedOption};
use crate::util::io;
use std::path::{Path, PathBuf};

const ESSENTIAL_DIRECTORY: &str = "essential";
const ESSENTIAL_CONFIG_FILE: &str = "config.toml";

pub(super) fn essential_settings_path(state: &State) -> PathBuf {
    synced_options_path(state)
        .join(ESSENTIAL_DIRECTORY)
        .join(ESSENTIAL_CONFIG_FILE)
}

pub(super) fn instance_essential_settings_path(
    metadata: &InstanceMetadata,
    state: &State,
) -> PathBuf {
    instance_dir(metadata, state)
        .join(ESSENTIAL_DIRECTORY)
        .join(ESSENTIAL_CONFIG_FILE)
}

/// Whether the instance has Essential, judged by the folder Essential creates on
/// first launch or an Essential jar in the mods folder.
async fn uses_essential(instance_dir: &Path) -> bool {
    if tokio::fs::metadata(instance_dir.join(ESSENTIAL_DIRECTORY))
        .await
        .is_ok_and(|entry| entry.is_dir())
    {
        return true;
    }
    let Ok(mut entries) = tokio::fs::read_dir(instance_dir.join("mods")).await
    else {
        return false;
    };
    while let Ok(Some(entry)) = entries.next_entry().await {
        let name = entry.file_name().to_string_lossy().to_lowercase();
        if name.starts_with("essential") && name.ends_with(".jar") {
            return true;
        }
    }
    false
}

/// Links the instance's Essential config to the shared copy.
///
/// The shared copy is created from the instance when it does not exist yet.
/// When `back_up_local` is set, a differing local config is backed up before
/// it is replaced.
pub(super) async fn ensure_essential_settings(
    metadata: &InstanceMetadata,
    back_up_local: bool,
    state: &State,
) -> crate::Result<()> {
    if !uses_essential(&instance_dir(metadata, state)).await {
        return Ok(());
    }
    let canonical = essential_settings_path(state);
    let local = instance_essential_settings_path(metadata, state);
    let local_is_link = tokio::fs::symlink_metadata(&local)
        .await
        .is_ok_and(|entry| entry.file_type().is_symlink());

    if !canonical.exists() {
        if !local.exists() {
            return Ok(());
        }
        let contents = io::read(&local).await?;
        if let Some(parent) = canonical.parent() {
            io::create_dir_all(parent).await?;
        }
        io::write(&canonical, contents).await?;
    }

    let canonical_bytes = io::read(&canonical).await?;
    if back_up_local && !local_is_link && local.exists() {
        let local_bytes = io::read(&local).await?;
        if local_bytes != canonical_bytes {
            backup_bytes(
                &metadata.instance.id,
                ESSENTIAL_CONFIG_FILE,
                &local_bytes,
                state,
            )
            .await?;
        }
    }

    begin_checkpoint(
        &metadata.instance.id,
        SyncedOption::EssentialSettings,
        "default",
        &sha1_bytes(&canonical_bytes),
        None,
        0,
        state,
    )
    .await?;
    let mode = ensure_link(&canonical, &local).await?;
    finish_checkpoint(
        &metadata.instance.id,
        SyncedOption::EssentialSettings,
        "default",
        mode,
        state,
    )
    .await
}

/// Picks up changes Essential made while the instance was linked.
///
/// Essential may replace the file instead of writing through the link, so a
/// regular file that no longer matches the last linked contents is treated as
/// the newest settings and copied to every participating instance.
pub(super) async fn reconcile_essential_settings(
    metadata: &InstanceMetadata,
    state: &State,
) -> crate::Result<()> {
    if !option_effective(metadata, SyncedOption::EssentialSettings, state)
        .await?
    {
        return Ok(());
    }
    let local = instance_essential_settings_path(metadata, state);
    let current_checkpoint = checkpoint(
        &metadata.instance.id,
        SyncedOption::EssentialSettings,
        "default",
        state,
    )
    .await?;
    let Some(current_checkpoint) = current_checkpoint else {
        return ensure_essential_settings(metadata, true, state).await;
    };
    if current_checkpoint.status == CheckpointStatus::Pending {
        return ensure_essential_settings(metadata, true, state).await;
    }
    if !local.exists() {
        return ensure_essential_settings(metadata, false, state).await;
    }

    let local_is_link = tokio::fs::symlink_metadata(&local)
        .await
        .is_ok_and(|entry| entry.file_type().is_symlink());
    if !local_is_link
        && sha1_file(&local).await? != current_checkpoint.expected_sha1
    {
        let canonical = essential_settings_path(state);
        if let Some(parent) = canonical.parent() {
            io::create_dir_all(parent).await?;
        }
        io::write(&canonical, io::read(&local).await?).await?;
        return refresh_essential_settings_links(state).await;
    }
    ensure_essential_settings(metadata, false, state).await
}

async fn refresh_essential_settings_links(state: &State) -> crate::Result<()> {
    for metadata in crate::state::list_instances(&state.pool).await? {
        if option_effective(&metadata, SyncedOption::EssentialSettings, state)
            .await?
        {
            ensure_essential_settings(&metadata, false, state).await?;
        }
    }
    Ok(())
}

/// Copies the instance's Essential config to the shared copy.
pub(super) async fn seed_essential_settings(
    metadata: &InstanceMetadata,
    state: &State,
) -> crate::Result<()> {
    let local = instance_essential_settings_path(metadata, state);
    if !local.exists() {
        return Ok(());
    }
    let canonical = essential_settings_path(state);
    if let Some(parent) = canonical.parent() {
        io::create_dir_all(parent).await?;
    }
    io::write(&canonical, io::read(&local).await?).await?;
    Ok(())
}

/// Gives the instance its own copy of the config, breaking any link.
pub(super) async fn detach_essential_settings(
    metadata: &InstanceMetadata,
    state: &State,
) -> crate::Result<()> {
    let local = instance_essential_settings_path(metadata, state);
    if tokio::fs::symlink_metadata(&local).await.is_err() {
        return Ok(());
    }
    let contents = if local.exists() {
        Some(io::read(&local).await?)
    } else {
        None
    };
    io::remove_file(&local).await?;
    if let Some(contents) = contents {
        io::write(&local, contents).await?;
    }
    Ok(())
}
