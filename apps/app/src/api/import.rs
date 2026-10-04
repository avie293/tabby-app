use std::path::PathBuf;

use crate::api::Result;
use theseus::install::InstallJobSnapshot;
use theseus::pack::import::ImportLauncherType;
use theseus::pack::import::modrinth_app::{
    ModrinthAppImportHistory, ModrinthAppImportOptions, ModrinthAppInstance,
};

use theseus::pack::import;

pub fn init<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::new("import")
        .invoke_handler(tauri::generate_handler![
            get_importable_instances,
            is_valid_importable_instance,
            get_default_launcher_path,
            modrinth_app_list_instances,
            modrinth_app_import_instances,
            modrinth_app_transfer_playtime,
            modrinth_app_get_history,
        ])
        .build()
}

/// Lists the instances of an installed official Modrinth App
#[tauri::command]
pub async fn modrinth_app_list_instances() -> Result<Vec<ModrinthAppInstance>> {
    Ok(import::modrinth_app::list_instances().await?)
}

/// Imports the selected Modrinth App instances with the selected data
#[tauri::command]
pub async fn modrinth_app_import_instances(
    instance_ids: Vec<String>,
    options: ModrinthAppImportOptions,
) -> Result<Vec<InstallJobSnapshot>> {
    Ok(import::modrinth_app::import_instances(&instance_ids, options).await?)
}

/// Adds a Modrinth App instance's playtime to an instance in this app
#[tauri::command]
pub async fn modrinth_app_transfer_playtime(
    source_instance_id: String,
    target_instance_id: String,
) -> Result<u64> {
    Ok(import::modrinth_app::transfer_playtime(
        &source_instance_id,
        &target_instance_id,
    )
    .await?)
}

/// Returns which Modrinth App instances and playtimes were already imported
#[tauri::command]
pub async fn modrinth_app_get_history() -> Result<ModrinthAppImportHistory> {
    Ok(import::modrinth_app::get_history().await?)
}

/// Gets a list of importable instances from a launcher type and base path
/// eg: get_importable_instances(ImportLauncherType::MultiMC, PathBuf::from("C:/MultiMC"))
/// returns ["Instance 1", "Instance 2"]
#[tauri::command]
pub async fn get_importable_instances(
    launcher_type: ImportLauncherType,
    base_path: PathBuf,
) -> Result<Vec<String>> {
    Ok(import::get_importable_instances(launcher_type, base_path).await?)
}

/// Checks if this instance is valid for importing, given a certain launcher type
/// eg: is_valid_importable_instance(PathBuf::from("C:/MultiMC/Instance 1"), ImportLauncherType::MultiMC)
#[tauri::command]
pub async fn is_valid_importable_instance(
    instance_folder: PathBuf,
    launcher_type: ImportLauncherType,
) -> Result<bool> {
    Ok(
        import::is_valid_importable_instance(instance_folder, launcher_type)
            .await,
    )
}

/// Returns the default path for the given launcher type
/// None if it can't be found or doesn't exist
#[tauri::command]
pub async fn get_default_launcher_path(
    launcher_type: ImportLauncherType,
) -> Result<Option<PathBuf>> {
    Ok(import::get_default_launcher_path(launcher_type))
}
