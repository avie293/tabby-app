use crate::api::Result;
use theseus::local_servers::{
    self, CreateLocalServer, EditLocalServer, LocalServer, LocalServerConsole,
    LocalServerInfo,
};

pub fn init<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::new("local-servers")
        .invoke_handler(tauri::generate_handler![
            local_server_list,
            local_server_get,
            local_server_create,
            local_server_edit,
            local_server_delete,
            local_server_start,
            local_server_stop,
            local_server_send_command,
            local_server_console,
        ])
        .build()
}

#[tauri::command]
pub async fn local_server_list() -> Result<Vec<LocalServerInfo>> {
    Ok(local_servers::list().await?)
}

#[tauri::command]
pub async fn local_server_get(id: String) -> Result<LocalServerInfo> {
    Ok(local_servers::get(&id).await?)
}

#[tauri::command]
pub async fn local_server_create(
    request: CreateLocalServer,
) -> Result<LocalServer> {
    Ok(local_servers::create(request).await?)
}

#[tauri::command]
pub async fn local_server_edit(
    id: String,
    patch: EditLocalServer,
) -> Result<LocalServer> {
    Ok(local_servers::edit(&id, patch).await?)
}

#[tauri::command]
pub async fn local_server_delete(id: String) -> Result<()> {
    Ok(local_servers::delete(&id).await?)
}

#[tauri::command]
pub async fn local_server_start(id: String) -> Result<()> {
    Ok(local_servers::start(&id).await?)
}

#[tauri::command]
pub async fn local_server_stop(id: String) -> Result<()> {
    Ok(local_servers::stop(&id).await?)
}

#[tauri::command]
pub async fn local_server_send_command(
    id: String,
    command: String,
) -> Result<()> {
    Ok(local_servers::send_command(&id, &command).await?)
}

#[tauri::command]
pub async fn local_server_console(
    id: String,
    after: Option<u64>,
) -> Result<LocalServerConsole> {
    Ok(local_servers::console(&id, after).await?)
}
