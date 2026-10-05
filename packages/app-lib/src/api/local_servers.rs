//! Minecraft servers hosted on this computer.
//!
//! Each server lives in its own folder below `<app dir>/servers`, next to a
//! `servers.json` registry. Running servers are child processes of the app;
//! their console output is buffered so the UI can poll it.

use std::collections::{HashMap, VecDeque};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::{Arc, LazyLock};
use std::time::Duration;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{ChildStdin, Command};
use tokio::sync::Mutex;

use crate::state::ModLoader;
use crate::util::fetch::fetch;
use crate::util::io;
use crate::{ErrorKind, State};

const SERVERS_FOLDER: &str = "servers";
const REGISTRY_FILE: &str = "servers.json";
const SERVER_JAR: &str = "server.jar";
const CONSOLE_LIMIT: usize = 2000;
const STOP_TIMEOUT: Duration = Duration::from_secs(30);
const DEFAULT_PORT: u16 = 25565;

static REGISTRY_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));
static RUNNING: LazyLock<Mutex<HashMap<String, RunningServer>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ServerSoftware {
    Vanilla,
    Fabric,
    Paper,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LocalServer {
    pub id: String,
    pub name: String,
    pub software: ServerSoftware,
    pub game_version: String,
    pub loader_version: Option<String>,
    pub java_major: u32,
    pub memory_mb: u32,
    pub port: u16,
    pub created: DateTime<Utc>,
}

#[derive(Serialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LocalServerStatus {
    Stopped,
    Starting,
    Running,
    Stopping,
}

#[derive(Serialize, Clone, Debug)]
pub struct LocalServerInfo {
    #[serde(flatten)]
    pub server: LocalServer,
    pub status: LocalServerStatus,
    pub path: PathBuf,
}

#[derive(Serialize, Clone, Debug)]
pub struct ConsoleLine {
    pub seq: u64,
    pub text: String,
}

#[derive(Serialize, Clone, Debug)]
pub struct LocalServerConsole {
    pub status: LocalServerStatus,
    pub lines: Vec<ConsoleLine>,
}

#[derive(Deserialize, Debug)]
pub struct CreateLocalServer {
    pub name: String,
    pub software: ServerSoftware,
    pub game_version: String,
    pub memory_mb: u32,
    pub accept_eula: bool,
}

#[derive(Deserialize, Debug, Default)]
pub struct EditLocalServer {
    pub name: Option<String>,
    pub memory_mb: Option<u32>,
    pub port: Option<u16>,
}

#[derive(Default)]
struct ConsoleBuffer {
    next_seq: u64,
    lines: VecDeque<ConsoleLine>,
}

impl ConsoleBuffer {
    fn push(&mut self, text: String) {
        self.lines.push_back(ConsoleLine {
            seq: self.next_seq,
            text,
        });
        self.next_seq += 1;
        while self.lines.len() > CONSOLE_LIMIT {
            self.lines.pop_front();
        }
    }
}

struct RunningServer {
    stdin: Arc<Mutex<ChildStdin>>,
    console: Arc<Mutex<ConsoleBuffer>>,
    status: Arc<Mutex<LocalServerStatus>>,
}

fn servers_dir(state: &State) -> PathBuf {
    state.directories.config_dir.join(SERVERS_FOLDER)
}

fn server_dir(state: &State, id: &str) -> PathBuf {
    servers_dir(state).join(id)
}

async fn read_registry(state: &State) -> crate::Result<Vec<LocalServer>> {
    let path = servers_dir(state).join(REGISTRY_FILE);
    if !path.exists() {
        return Ok(Vec::new());
    }
    Ok(serde_json::from_slice(&io::read(&path).await?)?)
}

async fn write_registry(
    state: &State,
    servers: &[LocalServer],
) -> crate::Result<()> {
    io::create_dir_all(servers_dir(state)).await?;
    io::write(
        servers_dir(state).join(REGISTRY_FILE),
        serde_json::to_vec_pretty(servers)?,
    )
    .await?;
    Ok(())
}

async fn find_server(state: &State, id: &str) -> crate::Result<LocalServer> {
    read_registry(state)
        .await?
        .into_iter()
        .find(|server| server.id == id)
        .ok_or_else(|| {
            ErrorKind::InputError("Unknown local server".to_string()).into()
        })
}

async fn status_of(id: &str) -> LocalServerStatus {
    let running = RUNNING.lock().await;
    match running.get(id) {
        Some(server) => *server.status.lock().await,
        None => LocalServerStatus::Stopped,
    }
}

pub async fn list() -> crate::Result<Vec<LocalServerInfo>> {
    let state = State::get().await?;
    let servers = read_registry(&state).await?;
    let mut infos = Vec::with_capacity(servers.len());
    for server in servers {
        infos.push(LocalServerInfo {
            status: status_of(&server.id).await,
            path: server_dir(&state, &server.id),
            server,
        });
    }
    Ok(infos)
}

pub async fn get(id: &str) -> crate::Result<LocalServerInfo> {
    let state = State::get().await?;
    let server = find_server(&state, id).await?;
    Ok(LocalServerInfo {
        status: status_of(id).await,
        path: server_dir(&state, id),
        server,
    })
}

/// Downloads the server software and prepares a new server folder.
///
/// The Minecraft EULA has to be accepted by the user before the server can be
/// created, because creating it accepts the EULA in `eula.txt`.
pub async fn create(request: CreateLocalServer) -> crate::Result<LocalServer> {
    if !request.accept_eula {
        return Err(ErrorKind::InputError(
            "You need to accept the Minecraft EULA to create a server."
                .to_string(),
        )
        .into());
    }
    let name = request.name.trim().to_string();
    if name.is_empty() {
        return Err(ErrorKind::InputError(
            "Give your server a name.".to_string(),
        )
        .into());
    }

    let state = State::get().await?;
    let (manifest, index) = crate::launcher::resolve_minecraft_manifest(
        &request.game_version,
        &state,
    )
    .await?;
    let version_info = crate::launcher::download::download_version_info(
        &state,
        &manifest.versions[index],
        None,
        None,
        None,
        None,
    )
    .await?;
    let java_major = version_info
        .java_version
        .as_ref()
        .map_or(8, |java| java.major_version);

    let (jar_url, jar_sha1, loader_version) = match request.software {
        ServerSoftware::Vanilla => {
            let download = version_info
                .downloads
                .get(&daedalus::minecraft::DownloadType::Server)
                .ok_or_else(|| {
                    ErrorKind::InputError(format!(
                        "Minecraft {} has no official server download.",
                        request.game_version
                    ))
                })?;
            (download.url.clone(), Some(download.sha1.clone()), None)
        }
        ServerSoftware::Fabric => {
            let loader = crate::launcher::get_loader_version_from_profile(
                &request.game_version,
                ModLoader::Fabric,
                Some("stable"),
            )
            .await?
            .ok_or_else(|| {
                ErrorKind::InputError(format!(
                    "Fabric is not available for Minecraft {}.",
                    request.game_version
                ))
            })?;
            let installer = latest_fabric_installer(&state).await?;
            (
                format!(
                    "https://meta.fabricmc.net/v2/versions/loader/{}/{}/{}/server/jar",
                    request.game_version, loader.id, installer
                ),
                None,
                Some(loader.id),
            )
        }
        ServerSoftware::Paper => {
            let (url, build) =
                latest_paper_build(&state, &request.game_version).await?;
            (url, None, Some(build))
        }
    };

    let id = uuid::Uuid::new_v4().to_string();
    let directory = server_dir(&state, &id);
    io::create_dir_all(&directory).await?;
    let result = async {
        let jar = fetch(
            &jar_url,
            jar_sha1.as_deref(),
            None,
            None,
            &state.fetch_semaphore,
            &state.pool,
        )
        .await?;
        io::write(directory.join(SERVER_JAR), &jar).await?;
        io::write(
            directory.join("eula.txt"),
            "# Accepted in Tabbyapp: https://aka.ms/MinecraftEULA\neula=true\n",
        )
        .await?;
        Ok::<_, crate::Error>(())
    }
    .await;
    if let Err(error) = result {
        let _ = io::remove_dir_all(&directory).await;
        return Err(error);
    }

    let _guard = REGISTRY_LOCK.lock().await;
    let mut servers = read_registry(&state).await?;
    let port = free_default_port(&servers);
    let server = LocalServer {
        id,
        name,
        software: request.software,
        game_version: request.game_version,
        loader_version,
        java_major,
        memory_mb: request.memory_mb.clamp(512, 65536),
        port,
        created: Utc::now(),
    };
    write_server_properties(&directory, &server).await?;
    servers.push(server.clone());
    write_registry(&state, &servers).await?;
    Ok(server)
}

fn free_default_port(servers: &[LocalServer]) -> u16 {
    let mut port = DEFAULT_PORT;
    while servers.iter().any(|server| server.port == port) {
        port += 1;
    }
    port
}

async fn latest_fabric_installer(state: &State) -> crate::Result<String> {
    #[derive(Deserialize)]
    struct Installer {
        version: String,
        stable: bool,
    }
    let installers: Vec<Installer> = crate::util::fetch::fetch_json(
        reqwest::Method::GET,
        "https://meta.fabricmc.net/v2/versions/installer",
        None,
        None,
        None,
        &state.fetch_semaphore,
        &state.pool,
    )
    .await?;
    installers
        .into_iter()
        .find(|installer| installer.stable)
        .map(|installer| installer.version)
        .ok_or_else(|| {
            ErrorKind::OtherError(
                "Could not find a Fabric server installer.".to_string(),
            )
            .into()
        })
}

async fn latest_paper_build(
    state: &State,
    game_version: &str,
) -> crate::Result<(String, String)> {
    #[derive(Deserialize)]
    struct Build {
        id: u64,
        downloads: HashMap<String, BuildDownload>,
    }
    #[derive(Deserialize)]
    struct BuildDownload {
        url: String,
    }
    let build: Build = crate::util::fetch::fetch_json(
        reqwest::Method::GET,
        &format!(
            "https://fill.papermc.io/v3/projects/paper/versions/{game_version}/builds/latest"
        ),
        None,
        None,
        None,
        &state.fetch_semaphore,
        &state.pool,
    )
    .await
    .map_err(|_| {
        ErrorKind::InputError(format!(
            "Paper is not available for Minecraft {game_version}."
        ))
    })?;
    let download = build.downloads.get("server:default").ok_or_else(|| {
        ErrorKind::InputError(format!(
            "Paper is not available for Minecraft {game_version}."
        ))
    })?;
    Ok((download.url.clone(), build.id.to_string()))
}

/// Writes the port and MOTD into `server.properties`, keeping every other
/// setting the user changed.
async fn write_server_properties(
    directory: &Path,
    server: &LocalServer,
) -> crate::Result<()> {
    let path = directory.join("server.properties");
    let existing = if path.exists() {
        String::from_utf8_lossy(&io::read(&path).await?).into_owned()
    } else {
        String::new()
    };
    let mut values = vec![
        ("server-port", server.port.to_string()),
        ("query.port", server.port.to_string()),
    ];
    if !existing.lines().any(|line| line.starts_with("motd=")) {
        values.push(("motd", server.name.clone()));
    }
    let mut lines = existing.lines().map(str::to_string).collect::<Vec<_>>();
    for (key, value) in values {
        let entry = format!("{key}={value}");
        match lines
            .iter_mut()
            .find(|line| line.split_once('=').is_some_and(|(k, _)| k == key))
        {
            Some(line) => *line = entry,
            None => lines.push(entry),
        }
    }
    io::write(&path, lines.join("\n") + "\n").await?;
    Ok(())
}

async fn java_path(java_major: u32) -> crate::Result<PathBuf> {
    if let Some(java) = crate::api::jre::get_java_versions()
        .await?
        .get(&java_major)
        .map(|java| PathBuf::from(&java.path))
        && java.exists()
    {
        return Ok(java);
    }
    crate::api::jre::auto_install_java(java_major).await
}

pub async fn start(id: &str) -> crate::Result<()> {
    let state = State::get().await?;
    let server = find_server(&state, id).await?;
    let directory = server_dir(&state, id);
    {
        let running = RUNNING.lock().await;
        if running.contains_key(id) {
            return Ok(());
        }
    }
    write_server_properties(&directory, &server).await?;
    let java = java_path(server.java_major).await?;

    let mut command = Command::new(&java);
    command
        .current_dir(&directory)
        .arg(format!("-Xmx{}M", server.memory_mb))
        .arg(format!("-Xms{}M", server.memory_mb.min(1024)))
        .args(["-jar", SERVER_JAR, "nogui"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    #[cfg(windows)]
    {
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        command.creation_flags(CREATE_NO_WINDOW);
    }
    let mut child = command.spawn()?;

    let console = Arc::new(Mutex::new(ConsoleBuffer::default()));
    let status = Arc::new(Mutex::new(LocalServerStatus::Starting));
    let stdin = child.stdin.take().ok_or_else(|| {
        ErrorKind::OtherError("The server has no input stream".to_string())
    })?;
    if let Some(stdout) = child.stdout.take() {
        spawn_reader(stdout, console.clone(), status.clone());
    }
    if let Some(stderr) = child.stderr.take() {
        spawn_reader(stderr, console.clone(), status.clone());
    }
    console.lock().await.push(format!(
        "[Tabbyapp] Starting {} with Java {} and {} MB of memory",
        server.name, server.java_major, server.memory_mb
    ));

    RUNNING.lock().await.insert(
        id.to_string(),
        RunningServer {
            stdin: Arc::new(Mutex::new(stdin)),
            console: console.clone(),
            status,
        },
    );
    let id = id.to_string();
    tokio::spawn(async move {
        let exit = child.wait().await;
        let message = match exit {
            Ok(code) => format!("[Tabbyapp] Server stopped ({code})"),
            Err(error) => format!("[Tabbyapp] Server stopped: {error}"),
        };
        console.lock().await.push(message);
        let mut running = RUNNING.lock().await;
        if let Some(server) = running.remove(&id) {
            *server.status.lock().await = LocalServerStatus::Stopped;
            EXITED.lock().await.insert(id, server.console);
        }
    });
    Ok(())
}

/// Console output of servers that already stopped, so the last messages stay
/// visible until the server is started again.
static EXITED: LazyLock<Mutex<HashMap<String, Arc<Mutex<ConsoleBuffer>>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

fn spawn_reader(
    stream: impl tokio::io::AsyncRead + Unpin + Send + 'static,
    console: Arc<Mutex<ConsoleBuffer>>,
    status: Arc<Mutex<LocalServerStatus>>,
) {
    tokio::spawn(async move {
        let mut lines = BufReader::new(stream).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            if line.contains("]: Done (") || line.contains(" Done (") {
                let mut status = status.lock().await;
                if *status == LocalServerStatus::Starting {
                    *status = LocalServerStatus::Running;
                }
            }
            console.lock().await.push(line);
        }
    });
}

pub async fn send_command(id: &str, command: &str) -> crate::Result<()> {
    let (stdin, console) = {
        let running = RUNNING.lock().await;
        let server = running.get(id).ok_or_else(|| {
            ErrorKind::InputError("This server is not running".to_string())
        })?;
        (server.stdin.clone(), server.console.clone())
    };
    let command = command.trim();
    if command.is_empty() {
        return Ok(());
    }
    console.lock().await.push(format!("> {command}"));
    let mut stdin = stdin.lock().await;
    stdin.write_all(format!("{command}\n").as_bytes()).await?;
    stdin.flush().await?;
    Ok(())
}

pub async fn stop(id: &str) -> crate::Result<()> {
    let status = {
        let running = RUNNING.lock().await;
        let Some(server) = running.get(id) else {
            return Ok(());
        };
        server.status.clone()
    };
    *status.lock().await = LocalServerStatus::Stopping;
    send_command(id, "stop").await
}

/// Asks every running server to stop and waits for them to save their worlds.
/// Servers that do not stop in time are killed when the app exits.
pub async fn stop_all() {
    let ids = RUNNING.lock().await.keys().cloned().collect::<Vec<_>>();
    for id in &ids {
        let _ = stop(id).await;
    }
    let deadline = tokio::time::Instant::now() + STOP_TIMEOUT;
    while !RUNNING.lock().await.is_empty()
        && tokio::time::Instant::now() < deadline
    {
        tokio::time::sleep(Duration::from_millis(250)).await;
    }
}

pub async fn console(
    id: &str,
    after: Option<u64>,
) -> crate::Result<LocalServerConsole> {
    let status = status_of(id).await;
    let buffer = {
        let running = RUNNING.lock().await;
        match running.get(id) {
            Some(server) => Some(server.console.clone()),
            None => EXITED.lock().await.get(id).cloned(),
        }
    };
    let lines = match buffer {
        Some(buffer) => buffer
            .lock()
            .await
            .lines
            .iter()
            .filter(|line| after.is_none_or(|after| line.seq > after))
            .cloned()
            .collect(),
        None => Vec::new(),
    };
    Ok(LocalServerConsole { status, lines })
}

pub async fn edit(
    id: &str,
    patch: EditLocalServer,
) -> crate::Result<LocalServer> {
    let state = State::get().await?;
    let _guard = REGISTRY_LOCK.lock().await;
    let mut servers = read_registry(&state).await?;
    let server = servers
        .iter_mut()
        .find(|server| server.id == id)
        .ok_or_else(|| {
            ErrorKind::InputError("Unknown local server".to_string())
        })?;
    if let Some(name) = patch.name.map(|name| name.trim().to_string())
        && !name.is_empty()
    {
        server.name = name;
    }
    if let Some(memory_mb) = patch.memory_mb {
        server.memory_mb = memory_mb.clamp(512, 65536);
    }
    if let Some(port) = patch.port {
        server.port = port;
    }
    let server = server.clone();
    write_server_properties(&server_dir(&state, id), &server).await?;
    write_registry(&state, &servers).await?;
    Ok(server)
}

pub async fn delete(id: &str) -> crate::Result<()> {
    if status_of(id).await != LocalServerStatus::Stopped {
        return Err(ErrorKind::InputError(
            "Stop the server before deleting it".to_string(),
        )
        .into());
    }
    let state = State::get().await?;
    let _guard = REGISTRY_LOCK.lock().await;
    let mut servers = read_registry(&state).await?;
    servers.retain(|server| server.id != id);
    write_registry(&state, &servers).await?;
    let directory = server_dir(&state, id);
    if directory.exists() {
        io::remove_dir_all(&directory).await?;
    }
    EXITED.lock().await.remove(id);
    Ok(())
}
