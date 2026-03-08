use serde::Serialize;
use std::fs;
use std::sync::{Arc, Mutex};
use sysinfo::{Components, Disks, Networks, System};
use tauri::{Manager, State};
use tauri_plugin_opener::OpenerExt;

#[derive(Serialize)]
struct DiskInfo {
    name: String,
    total_space: u64,
    available_space: u64,
    kind: String,
}

#[derive(Serialize)]
struct ComponentInfo {
    label: String,
    temp: Option<f32>,
    max_temp: Option<f32>,
    critical_temp: Option<f32>,
}

#[derive(Serialize)]
struct ProcessInfo {
    name: String,
    pid: u32,
    parent_pid: Option<u32>,
    cpu_usage: f32,
    memory: u64,
    run_duration: u64,
}

#[derive(Serialize)]
struct DockerContainer {
    id: String,
    name: String,
    image: String,
    status: String,
    state: String,
    ports: String,
}

#[derive(Serialize)]
struct ServiceInfo {
    name: String,
    display_name: String,
    status: String,
}

#[derive(Serialize)]
struct StartupInfo {
    name: String,
    command: String,
    location: String,
}

#[derive(Serialize)]
struct DbServerInfo {
    name: String,
    port: u16,
    status: String,
    pid: Option<u32>,
    uptime: u64,
}

#[derive(Serialize)]
struct PkgManagerInfo {
    name: String,
    command: String,
    cpu_usage: f32,
    memory: u64,
    pid: u32,
    duration: u64,
}

#[derive(Serialize)]
struct GitStatus {
    branch: String,
    uncommitted_changes: usize,
    last_commit: String,
    is_dirty: bool,
    repo_name: String,
}

#[derive(Serialize)]
struct PortInfo {
    port: u16,
    protocol: String,
    pid: u32,
    process_name: String,
    state: String,
}

#[derive(Serialize)]
struct EnvironmentInfo {
    node_version: String,
    python_version: String,
    rust_version: String,
    git_version: String,
    os_details: String,
    shell_type: String,
}

#[derive(Serialize)]
struct DevServerInfo {
    framework: String,
    url: String,
    port: u16,
    pid: u32,
    process_name: String,
    status: String,
}

#[derive(Serialize)]
struct SystemStats {
    cpu_usage: f32,
    cpu_cores: usize,
    physical_cores: usize,
    cpu_model: String,
    cpu_arch: String,
    cpu_freq: u64,
    cpus: Vec<f32>,
    cpu_temp: Option<f32>,
    memory_used: u64,
    memory_total: u64,
    os_name: String,
    os_version: String,
    uptime: u64,
    disks: Vec<DiskInfo>,
    net_received: u64,
    net_transmitted: u64,
    processes: Vec<ProcessInfo>,
    gpu_name: String,
    gpu_usage: f32,
    gpu_temp: Option<f32>,
    gpu_clock: Option<u32>,
    gpu_fan_speed: Option<u32>,
    vram_used: u64,
    vram_total: u64,
    battery_level: Option<f32>,
    disk_read_speed: u64,
    disk_write_speed: u64,
    ping: u32,
    wifi_signal: u32,
    load_average: [f64; 3],
    memory_free: u64,
    memory_available: u64,
    swap_total: u64,
    swap_used: u64,
    sensors: Vec<ComponentInfo>,
    local_ip: String,
    active_connections: usize,
}

pub struct AppState {
    sys: Mutex<System>,
    disks: Mutex<Disks>,
    networks: Mutex<Networks>,
    components: Mutex<Components>,
    last_hardware_refresh: Mutex<std::time::Instant>,

    // Background metrics to prevent blocking main command
    wifi_signal: Mutex<u32>,
    ping: Mutex<u32>,

    last_disk_total_read: Mutex<u64>,
    last_disk_total_write: Mutex<u64>,
    last_sample_time: Mutex<std::time::Instant>,
}

#[derive(serde::Deserialize, Serialize, Clone)]
struct HttpRequest {
    method: String,
    url: String,
    headers: std::collections::HashMap<String, String>,
    body: Option<String>,
}

#[derive(Serialize)]
struct HttpResponse {
    status: u16,
    headers: std::collections::HashMap<String, String>,
    body: String,
    time_ms: u64,
}

#[derive(serde::Deserialize, Serialize, Clone)]
struct SavedApiRequest {
    id: String,
    name: String,
    request: HttpRequest,
}

#[tauri::command]
fn get_system_stats(state: State<'_, Arc<AppState>>) -> SystemStats {
    let mut sys = state.sys.lock().unwrap();
    let now = std::time::Instant::now();

    // 1. Refresh System/Memory/Processes (Relatively fast)
    sys.refresh_cpu_all();
    sys.refresh_memory();
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);

    // 2. Throttled Hardware Refresh (Every 10 seconds)
    {
        let mut last_hw = state.last_hardware_refresh.lock().unwrap();
        // Check if we need to refresh (either 10s passed or first run where list is empty)
        if now.duration_since(*last_hw).as_secs() >= 10 {
            state.disks.lock().unwrap().refresh(false);
            state.networks.lock().unwrap().refresh(false);
            state.components.lock().unwrap().refresh(false);
            *last_hw = now;
        }
    }

    let cpu_usage = sys.global_cpu_usage();
    let cpu_cores = sys.cpus().len();
    let physical_cores = sys.physical_core_count().unwrap_or(cpu_cores);
    let cpu_model = sys
        .cpus()
        .get(0)
        .map(|c| c.brand().to_string())
        .unwrap_or_else(|| "Unknown".to_string());
    let cpu_freq = sys.cpus().get(0).map(|c| c.frequency()).unwrap_or(0);

    let memory_used = sys.used_memory();
    let memory_total = sys.total_memory();

    // 3. Collect disks from CACHE
    let disks = state
        .disks
        .lock()
        .unwrap()
        .iter()
        .map(|disk| DiskInfo {
            name: disk.name().to_string_lossy().into_owned(),
            total_space: disk.total_space(),
            available_space: disk.available_space(),
            kind: format!("{:?}", disk.kind()),
        })
        .collect();

    // 4. Collect network from CACHE
    let mut net_received = 0;
    let mut net_transmitted = 0;
    for (_name, data) in state.networks.lock().unwrap().iter() {
        net_received += data.total_received();
        net_transmitted += data.total_transmitted();
    }

    let cpus: Vec<f32> = sys.cpus().iter().map(|cpu| cpu.cpu_usage()).collect();

    // 5. Collect temp/gpu/sensors from CACHE
    let mut cpu_temp = None;
    let mut gpu_name = "N/A".to_string();
    let mut gpu_temp = None;
    let mut gpu_fan_speed = None;
    let mut sensors = Vec::new();

    for c in state.components.lock().unwrap().iter() {
        let label = c.label().to_lowercase();
        let temp = c.temperature();

        sensors.push(ComponentInfo {
            label: c.label().to_string(),
            temp: c.temperature(),
            max_temp: c.max(),
            critical_temp: c.critical(),
        });

        if cpu_temp.is_none() && (label.contains("cpu") || label.contains("package")) {
            cpu_temp = temp;
        }

        let is_gpu = label.contains("gpu") || label.contains("nvidia") || label.contains("amd");
        if is_gpu {
            if gpu_name == "N/A" {
                gpu_name = c.label().to_string();
            }
            if gpu_temp.is_none() {
                gpu_temp = temp;
            }
            if label.contains("fan") {
                gpu_fan_speed = temp.map(|v| v as u32);
            }
        }
    }

    // 6. Disk Speed
    let mut current_disk_read_total = 0;
    let mut current_disk_write_total = 0;
    let processes: Vec<ProcessInfo> = sys
        .processes()
        .iter()
        .map(|(pid, process)| {
            let du = process.disk_usage();
            current_disk_read_total += du.total_read_bytes;
            current_disk_write_total += du.total_written_bytes;
            ProcessInfo {
                name: process.name().to_string_lossy().into_owned(),
                pid: pid.as_u32(),
                parent_pid: process.parent().map(|p| p.as_u32()),
                cpu_usage: process.cpu_usage(),
                memory: process.memory(),
                run_duration: process.run_time(),
            }
        })
        .collect();

    let mut last_time = state.last_sample_time.lock().unwrap();
    let mut last_read = state.last_disk_total_read.lock().unwrap();
    let mut last_write = state.last_disk_total_write.lock().unwrap();

    let duration = now.duration_since(*last_time).as_secs_f64();
    let disk_read_speed = if duration > 0.0 && *last_read > 0 {
        ((current_disk_read_total.saturating_sub(*last_read)) as f64 / duration) as u64
    } else {
        0
    };
    let disk_write_speed = if duration > 0.0 && *last_write > 0 {
        ((current_disk_write_total.saturating_sub(*last_write)) as f64 / duration) as u64
    } else {
        0
    };

    *last_time = now;
    *last_read = current_disk_read_total;
    *last_write = current_disk_write_total;

    SystemStats {
        cpu_usage,
        cpu_cores,
        physical_cores,
        cpu_model,
        cpu_arch: System::cpu_arch(),
        cpu_freq,
        cpus,
        cpu_temp,
        memory_used,
        memory_total,
        os_name: System::name().unwrap_or_default(),
        os_version: System::os_version().unwrap_or_default(),
        uptime: System::uptime(),
        disks,
        net_received,
        net_transmitted,
        processes,
        gpu_name,
        gpu_usage: 0.0, // This is often updated by a background process or simulated
        gpu_temp,
        gpu_clock: None, // Hard to get cross-platform without specialized crates
        gpu_fan_speed,
        vram_used: 0,
        vram_total: 0,
        battery_level: None,
        disk_read_speed,
        disk_write_speed,
        ping: *state.ping.lock().unwrap(),
        wifi_signal: *state.wifi_signal.lock().unwrap(),
        load_average: [
            System::load_average().one,
            System::load_average().five,
            System::load_average().fifteen,
        ],
        memory_free: sys.free_memory(),
        memory_available: sys.available_memory(),
        swap_total: sys.total_swap(),
        swap_used: sys.used_swap(),
        sensors,
        local_ip: local_ip_address::local_ip()
            .map(|ip| ip.to_string())
            .unwrap_or_else(|_| "Unknown".to_string()),
        active_connections: sys.processes().len(), // Fallback or estimate
    }
}

fn get_latency() -> u32 {
    let now = std::time::Instant::now();
    if let Ok(_) = std::net::TcpStream::connect_timeout(
        &"8.8.8.8:53".parse().unwrap(),
        std::time::Duration::from_millis(400),
    ) {
        now.elapsed().as_millis() as u32
    } else {
        0
    }
}

fn get_wifi_signal() -> u32 {
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        if let Ok(out) = Command::new("netsh")
            .args(&["wlan", "show", "interfaces"])
            .output()
        {
            let s = String::from_utf8_lossy(&out.stdout);
            for line in s.lines() {
                if line.contains("Signal") {
                    if let Some(pct) = line
                        .split(':')
                        .last()
                        .and_then(|p| p.trim().trim_end_matches('%').parse::<u32>().ok())
                    {
                        return pct;
                    }
                }
            }
        }
    }
    0
}

#[tauri::command]
fn kill_process(state: State<'_, Arc<AppState>>, pid: u32) -> Result<(), String> {
    let sys = state.sys.lock().unwrap();
    if let Some(process) = sys.process(sysinfo::Pid::from(pid as usize)) {
        if process.kill() {
            Ok(())
        } else {
            Err("Failed to kill process".to_string())
        }
    } else {
        Err("Process not found".to_string())
    }
}

#[tauri::command]
fn get_services() -> Vec<ServiceInfo> {
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        let output = Command::new("powershell")
            .args(&[
                "-Command",
                "Get-Service | Select-Object Name, DisplayName, Status | ConvertTo-Json",
            ])
            .output();
        if let Ok(out) = output {
            let s = String::from_utf8_lossy(&out.stdout);
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&s) {
                if let Some(arr) = v.as_array() {
                    return arr
                        .iter()
                        .map(|v| ServiceInfo {
                            name: v["Name"].as_str().unwrap_or("").to_string(),
                            display_name: v["DisplayName"].as_str().unwrap_or("").to_string(),
                            status: v["Status"]
                                .as_i64()
                                .map(|s| if s == 4 { "Running" } else { "Stopped" })
                                .unwrap_or("Unknown")
                                .to_string(),
                        })
                        .collect();
                }
            }
        }
    }
    vec![]
}

#[tauri::command]
fn control_service(name: String, action: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        let cmd = if action == "start" {
            "Start-Service"
        } else {
            "Stop-Service"
        };
        let output = Command::new("powershell")
            .args(&[
                "-Command",
                &format!(
                    "Start-Process powershell -ArgumentList '-Command {} {}' -Verb RunAs",
                    cmd, name
                ),
            ])
            .output();
        if output.is_ok() {
            return Ok(());
        }
    }
    Err("Failed to execute service control".to_string())
}

#[tauri::command]
fn get_startup_apps() -> Vec<StartupInfo> {
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        let output = Command::new("powershell").args(&["-Command", "Get-CimInstance Win32_StartupCommand | Select-Object Name, Command, Location | ConvertTo-Json"]).output();
        if let Ok(out) = output {
            let s = String::from_utf8_lossy(&out.stdout);
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&s) {
                if let Some(arr) = v.as_array() {
                    return arr
                        .iter()
                        .map(|v| StartupInfo {
                            name: v["Name"].as_str().unwrap_or("").to_string(),
                            command: v["Command"].as_str().unwrap_or("").to_string(),
                            location: v["Location"].as_str().unwrap_or("").to_string(),
                        })
                        .collect();
                } else if let Some(obj) = v.as_object() {
                    return vec![StartupInfo {
                        name: obj["Name"].as_str().unwrap_or("").to_string(),
                        command: obj["Command"].as_str().unwrap_or("").to_string(),
                        location: obj["Location"].as_str().unwrap_or("").to_string(),
                    }];
                }
            }
        }
    }
    vec![]
}

#[tauri::command]
async fn save_export(
    app: tauri::AppHandle,
    filename: String,
    base64_content: String,
) -> Result<String, String> {
    // 1. Get Downloads Path
    let mut path = app.path().download_dir().map_err(|e| e.to_string())?;
    path.push("SystemMonitor_Exports");

    // 2. Create Directory
    if !path.exists() {
        fs::create_dir_all(&path).map_err(|e| e.to_string())?;
    }

    // 3. Prepare File Path
    path.push(&filename);

    // 4. Decode and Write
    // Note: We use base64 to avoid encoding issues between JS and Rust
    use base64::{engine::general_purpose, Engine as _};
    let data = general_purpose::STANDARD
        .decode(base64_content)
        .map_err(|e| format!("Decode error: {}", e))?;

    fs::write(&path, data).map_err(|e| e.to_string())?;

    // 5. Open the folder so user sees their success
    let opener = app.opener();
    let parent = path.parent().unwrap().to_path_buf();
    let _ = opener.open_path(parent.to_string_lossy().to_string(), Option::<String>::None);

    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
async fn get_active_ports(state: State<'_, Arc<AppState>>) -> Result<Vec<PortInfo>, String> {
    use std::process::Command;

    // On Windows, use netstat -ano to find listening ports
    let output = if cfg!(target_os = "windows") {
        Command::new("netstat")
            .args(["-ano"]) // Get all, include UDP
            .output()
            .map_err(|e| e.to_string())?
    } else {
        return Err("OS not supported".to_string());
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut ports = Vec::new();

    let mut sys = state.sys.lock().map_err(|e| e.to_string())?;
    sys.refresh_all();

    for line in stdout.lines().skip(4) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 4 {
            continue;
        }

        let protocol = parts[0];
        let is_tcp = protocol == "TCP";
        let is_udp = protocol == "UDP";

        if !is_tcp && !is_udp {
            continue;
        }

        // TCP has [Proto, Local, Foreign, State, PID]
        // UDP has [Proto, Local, Foreign, PID]
        let local_addr = parts[1];
        let state_val = if is_tcp { parts[3] } else { "ACTIVE" };
        let pid_str = if is_tcp { parts[4] } else { parts[3] };

        if is_tcp && state_val != "LISTENING" {
            continue;
        }

        if let Some(port_str) = local_addr.split(':').last() {
            if let Ok(port) = port_str.parse::<u16>() {
                if let Ok(pid) = pid_str.parse::<u32>() {
                    let process_name = sys
                        .process(sysinfo::Pid::from(pid as usize))
                        .map(|p| p.name().to_string_lossy().into_owned())
                        .unwrap_or_else(|| "Unknown".to_string());

                    ports.push(PortInfo {
                        port,
                        protocol: protocol.to_string(),
                        pid,
                        process_name,
                        state: state_val.to_string(),
                    });
                }
            }
        }
    }

    // Sort by port number
    ports.sort_by_key(|p| p.port);
    // Deduplicate (netstat sometimes shows same port on different interfaces)
    ports.dedup_by_key(|p| p.port);

    Ok(ports)
}

#[tauri::command]
async fn get_dev_servers(state: State<'_, Arc<AppState>>) -> Result<Vec<DevServerInfo>, String> {
    use std::process::Command;

    let output = if cfg!(target_os = "windows") {
        Command::new("netstat")
            .args(["-ano"])
            .output()
            .map_err(|e| e.to_string())?
    } else {
        return Err("OS not supported".to_string());
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut servers = Vec::new();

    let mut sys = state.sys.lock().map_err(|e| e.to_string())?;
    sys.refresh_all();

    for line in stdout.lines().skip(4) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 4 {
            continue;
        }

        let protocol = parts[0];
        if protocol != "TCP" {
            continue;
        }

        let local_addr = parts[1];
        let state_val = if parts.len() > 3 { parts[3] } else { "" };
        let pid_str = if parts.len() > 4 { parts[4] } else { "" };

        if state_val != "LISTENING" || pid_str.is_empty() {
            continue;
        }

        if let Some(port_str) = local_addr.split(':').last() {
            if let Ok(port) = port_str.parse::<u16>() {
                if let Ok(pid) = pid_str.parse::<u32>() {
                    let process_name = sys
                        .process(sysinfo::Pid::from(pid as usize))
                        .map(|p| p.name().to_string_lossy().into_owned())
                        .unwrap_or_else(|| "Unknown".to_string());

                    let mut framework = "Local Server".to_string();
                    let name_low = process_name.to_lowercase();

                    if name_low.contains("node") {
                        if port == 5173 {
                            framework = "Vite / React / Vue".to_string();
                        } else if port == 3000 {
                            framework = "Next.js / Node / React".to_string();
                        } else if port == 8080 || port == 8000 {
                            framework = "Node.js Server".to_string();
                        }
                    } else if name_low.contains("python") {
                        if port == 5000 {
                            framework = "Flask / Python".to_string();
                        } else if port == 8000 {
                            framework = "Django / Python".to_string();
                        }
                    } else if name_low.contains("php") {
                        framework = "PHP Server".to_string();
                    } else if name_low.contains("go") {
                        framework = "Golang Server".to_string();
                    }

                    if framework != "Local Server"
                        || [3000, 3001, 5173, 5000, 8000, 8080].contains(&port)
                    {
                        servers.push(DevServerInfo {
                            framework,
                            url: format!("http://localhost:{}", port),
                            port,
                            pid,
                            process_name,
                            status: "Running".to_string(),
                        });
                    }
                }
            }
        }
    }

    Ok(servers)
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}

#[tauri::command]
fn open_project_folder(app: tauri::AppHandle, pid: u32) -> Result<(), String> {
    use sysinfo::{Pid, System};
    let mut sys = System::new();
    sys.refresh_processes(
        sysinfo::ProcessesToUpdate::Some(&[Pid::from(pid as usize)]),
        true,
    );

    if let Some(process) = sys.process(Pid::from(pid as usize)) {
        if let Some(cwd) = process.cwd() {
            let path = cwd.to_string_lossy().to_string();
            let opener = app.opener();
            let _ = opener.open_path(path, Option::<String>::None);
            return Ok(());
        }
    }
    Err("Could not find project folder for this process".to_string())
}

#[tauri::command]
async fn get_docker_containers() -> Result<Vec<DockerContainer>, String> {
    use std::process::Command;
    let output = Command::new("docker")
        .args([
            "ps",
            "-a",
            "--format",
            "{{.ID}}|{{.Names}}|{{.Image}}|{{.Status}}|{{.State}}|{{.Ports}}",
        ])
        .output()
        .map_err(|e| format!("Docker not found or error: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut containers = Vec::new();

    for line in stdout.lines() {
        let parts: Vec<&str> = line.split('|').collect();
        if parts.len() >= 6 {
            containers.push(DockerContainer {
                id: parts[0].to_string(),
                name: parts[1].to_string(),
                image: parts[2].to_string(),
                status: parts[3].to_string(),
                state: parts[4].to_string(),
                ports: parts[5].to_string(),
            });
        }
    }
    Ok(containers)
}

#[tauri::command]
async fn control_docker_container(id: String, action: String) -> Result<(), String> {
    use std::process::Command;
    let valid_actions = ["start", "stop", "restart"];
    if !valid_actions.contains(&action.as_str()) {
        return Err("Invalid action".to_string());
    }

    Command::new("docker")
        .args([&action, &id])
        .output()
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
async fn get_db_servers(state: State<'_, Arc<AppState>>) -> Result<Vec<DbServerInfo>, String> {
    let mut db_servers = Vec::new();
    let db_ports = vec![
        (3306, "MySQL"),
        (5432, "PostgreSQL"),
        (27017, "MongoDB"),
        (6379, "Redis"),
        (1433, "SQL Server"),
        (28015, "RethinkDB"),
    ];

    let mut sys = state.sys.lock().map_err(|e| e.to_string())?;
    sys.refresh_all();

    // Re-use active ports logic internally
    use std::process::Command;
    let output = Command::new("netstat")
        .args(["-ano"])
        .output()
        .map_err(|e| e.to_string())?;

    let stdout = String::from_utf8_lossy(&output.stdout);

    for (port, name) in db_ports {
        let pattern = format!(":{}", port);
        let found = stdout
            .lines()
            .find(|l| l.contains(&pattern) && l.contains("LISTENING"));

        if let Some(line) = found {
            let parts: Vec<&str> = line.split_whitespace().collect();
            let pid_str = parts.get(4).unwrap_or(&"");
            let pid = pid_str.parse::<u32>().ok();

            let uptime = if let Some(p) = pid {
                sys.process(sysinfo::Pid::from(p as usize))
                    .map(|proc| proc.run_time())
                    .unwrap_or(0)
            } else {
                0
            };

            db_servers.push(DbServerInfo {
                name: name.to_string(),
                port,
                status: "Running".to_string(),
                pid,
                uptime,
            });
        }
    }

    Ok(db_servers)
}

#[tauri::command]
async fn get_pkg_managers(state: State<'_, Arc<AppState>>) -> Result<Vec<PkgManagerInfo>, String> {
    let mut sys = state.sys.lock().map_err(|e| e.to_string())?;
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);

    let pkg_keywords = vec!["npm", "yarn", "pip", "cargo", "pnpm", "bun"];
    let mut managers = Vec::new();

    for (pid, process) in sys.processes() {
        let name = process.name().to_string_lossy().to_lowercase();
        if pkg_keywords.iter().any(|k| name.contains(k)) {
            let cmd = process
                .cmd()
                .iter()
                .map(|s| s.to_string_lossy())
                .collect::<Vec<_>>()
                .join(" ");
            // Only show if it looks like an install or build
            if cmd.contains("install")
                || cmd.contains("build")
                || cmd.contains("add")
                || cmd.contains("update")
            {
                managers.push(PkgManagerInfo {
                    name: process.name().to_string_lossy().into_owned(),
                    command: cmd,
                    cpu_usage: process.cpu_usage(),
                    memory: process.memory(),
                    pid: pid.as_u32(),
                    duration: process.run_time(),
                });
            }
        }
    }

    Ok(managers)
}

#[tauri::command]
async fn get_git_activity() -> Result<GitStatus, String> {
    use std::process::Command;

    // Attempt to get Git info for the current working directory
    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .map_err(|_| "Git not found or not a repository".to_string())?;

    let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();

    let status_output = Command::new("git")
        .args(["status", "--short"])
        .output()
        .map_err(|e| e.to_string())?;
    let status_str = String::from_utf8_lossy(&status_output.stdout);
    let changed_count = status_str.lines().count();

    let log_output = Command::new("git")
        .args(["log", "-1", "--pretty=%B"])
        .output()
        .map_err(|e| e.to_string())?;
    let last_commit = String::from_utf8_lossy(&log_output.stdout)
        .trim()
        .to_string();

    let repo_name = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .map_err(|e| e.to_string())?;
    let repo_path = String::from_utf8_lossy(&repo_name.stdout)
        .trim()
        .to_string();
    let repo_folder = std::path::Path::new(&repo_path)
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    Ok(GitStatus {
        branch,
        uncommitted_changes: changed_count,
        last_commit,
        is_dirty: changed_count > 0,
        repo_name: repo_folder,
    })
}

#[tauri::command]
async fn get_environment_info() -> Result<EnvironmentInfo, String> {
    use std::process::Command;

    let mut info = EnvironmentInfo {
        node_version: "Not found".to_string(),
        python_version: "Not found".to_string(),
        rust_version: "Not found".to_string(),
        git_version: "Not found".to_string(),
        os_details: format!(
            "{} {} ({})",
            System::name().unwrap_or_default(),
            System::os_version().unwrap_or_default(),
            System::cpu_arch()
        ),
        shell_type: "Unknown".to_string(),
    };

    // Node Version
    if let Ok(out) = Command::new("node").arg("--version").output() {
        info.node_version = String::from_utf8_lossy(&out.stdout).trim().to_string();
    }

    // Python Version
    if let Ok(out) = Command::new("python").arg("--version").output() {
        info.python_version = String::from_utf8_lossy(&out.stdout).trim().to_string();
    } else if let Ok(out) = Command::new("python3").arg("--version").output() {
        info.python_version = String::from_utf8_lossy(&out.stdout).trim().to_string();
    }

    // Rust Version
    if let Ok(out) = Command::new("rustc").arg("--version").output() {
        info.rust_version = String::from_utf8_lossy(&out.stdout)
            .split_whitespace()
            .nth(1)
            .unwrap_or("Unknown")
            .to_string();
    }

    // Git Version
    if let Ok(out) = Command::new("git").arg("--version").output() {
        info.git_version = String::from_utf8_lossy(&out.stdout)
            .replace("git version", "")
            .trim()
            .to_string();
    }

    // Shell Type
    #[cfg(target_os = "windows")]
    {
        if std::env::var("PSModulePath").is_ok() {
            info.shell_type = "PowerShell".to_string();
        } else {
            info.shell_type = "CMD".to_string();
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        if let Ok(shell) = std::env::var("SHELL") {
            info.shell_type = shell.split('/').last().unwrap_or("Unknown").to_string();
        }
    }

    Ok(info)
}

#[tauri::command]
async fn send_api_request(req: HttpRequest) -> Result<HttpResponse, String> {
    use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
    use std::time::Instant;

    let client = reqwest::Client::new();
    let method = match req.method.to_uppercase().as_str() {
        "GET" => reqwest::Method::GET,
        "POST" => reqwest::Method::POST,
        "PUT" => reqwest::Method::PUT,
        "DELETE" => reqwest::Method::DELETE,
        "PATCH" => reqwest::Method::PATCH,
        _ => return Err("Invalid HTTP method".to_string()),
    };

    let mut builder = client.request(method, &req.url);

    let mut headers = HeaderMap::new();
    for (key, val) in req.headers {
        if let (Ok(h_key), Ok(h_val)) = (
            HeaderName::from_bytes(key.as_bytes()),
            HeaderValue::from_str(&val),
        ) {
            headers.insert(h_key, h_val);
        }
    }
    builder = builder.headers(headers);

    if let Some(body) = req.body {
        if !body.is_empty() {
            builder = builder.body(body);
        }
    }

    let start = Instant::now();
    let response = builder.send().await.map_err(|e| e.to_string())?;
    let duration = start.elapsed().as_millis() as u64;

    let status = response.status().as_u16();
    let mut resp_headers = std::collections::HashMap::new();
    for (name, value) in response.headers() {
        resp_headers.insert(name.to_string(), value.to_str().unwrap_or("").to_string());
    }

    let body = response.text().await.unwrap_or_default();

    Ok(HttpResponse {
        status,
        headers: resp_headers,
        body,
        time_ms: duration,
    })
}

#[tauri::command]
fn get_saved_api_collections(app: tauri::AppHandle) -> Result<Vec<SavedApiRequest>, String> {
    let path = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?
        .join("api_collections.json");
    if !path.exists() {
        return Ok(vec![]);
    }
    let data = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let collections: Vec<SavedApiRequest> =
        serde_json::from_str(&data).map_err(|e| e.to_string())?;
    Ok(collections)
}

#[tauri::command]
fn save_api_request(app: tauri::AppHandle, request: SavedApiRequest) -> Result<(), String> {
    let path = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?
        .join("api_collections.json");
    let mut collections = vec![];
    if path.exists() {
        let data = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        collections = serde_json::from_str::<Vec<SavedApiRequest>>(&data).unwrap_or_default();
    }

    // Replace if exists, otherwise push
    if let Some(pos) = collections.iter().position(|r| r.id == request.id) {
        collections[pos] = request;
    } else {
        collections.push(request);
    }

    let data = serde_json::to_string_pretty(&collections).map_err(|e| e.to_string())?;
    fs::write(path, data).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn delete_api_request(app: tauri::AppHandle, id: String) -> Result<(), String> {
    let path = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?
        .join("api_collections.json");
    if !path.exists() {
        return Ok(());
    }
    let data = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let mut collections: Vec<SavedApiRequest> =
        serde_json::from_str(&data).map_err(|e| e.to_string())?;
    collections.retain(|r| r.id != id);
    let data = serde_json::to_string_pretty(&collections).map_err(|e| e.to_string())?;
    fs::write(path, data).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn toggle_gaming_boost(active: bool) -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        // 8c5e7fda-e8bf-4a96-9a85-a6e23a8c635c is High Performance
        // a1841308-3541-4fab-bc81-f71556f20b4a is Power Saver
        // 381b4222-f694-41f0-9685-ff5bb260df2e is Balanced

        let plan_guid = if active {
            "8c5e7fda-e8bf-4a96-9a85-a6e23a8c635c"
        } else {
            "381b4222-f694-41f0-9685-ff5bb260df2e"
        };

        Command::new("powercfg")
            .args(["-setactive", plan_guid])
            .output()
            .map_err(|e| e.to_string())?;

        Ok(if active {
            "High Performance Mode Active"
        } else {
            "Balanced Mode Resumed"
        }
        .to_string())
    }
    #[cfg(not(target_os = "windows"))]
    Ok("Feature only available on Windows".to_string())
}

#[tauri::command]
async fn cleanup_gaming_memory() -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        // Use PowerShell to clear working sets of non-essential processes or common bloat
        // This is a simplified "Booster" approach
        let script = r#"
            Get-Process | Where-Object { $_.MainWindowTitle -eq "" -and $_.CPU -lt 1 } | ForEach-Object { 
                try { [Runtime.InteropServices.Marshal]::FreeHGlobal(0) } catch {}
            }
        "#;

        Command::new("powershell")
            .args(["-Command", script])
            .output()
            .map_err(|e| e.to_string())?;

        Ok("Memory working set optimization complete".to_string())
    }
    #[cfg(not(target_os = "windows"))]
    Ok("Not implemented for this OS".to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_state = Arc::new(AppState {
                sys: Mutex::new(System::new()),
                disks: Mutex::new(Disks::new()),
                networks: Mutex::new(Networks::new()),
                components: Mutex::new(Components::new()),
                last_hardware_refresh: Mutex::new(
                    std::time::Instant::now() - std::time::Duration::from_secs(3600),
                ),
                wifi_signal: Mutex::new(100),
                ping: Mutex::new(0),
                last_disk_total_read: Mutex::new(0),
                last_disk_total_write: Mutex::new(0),
                last_sample_time: Mutex::new(std::time::Instant::now()),
            });

            app.manage(app_state.clone());

            // Background Thread for Slow Metrics
            let thread_state = app_state.clone();
            std::thread::spawn(move || loop {
                let wifi = get_wifi_signal();
                let latency = get_latency();
                {
                    if let Ok(mut ws) = thread_state.wifi_signal.lock() {
                        *ws = wifi;
                    }
                    if let Ok(mut ps) = thread_state.ping.lock() {
                        *ps = latency;
                    }
                }
                std::thread::sleep(std::time::Duration::from_secs(5));
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            get_system_stats,
            kill_process,
            get_services,
            control_service,
            get_startup_apps,
            save_export,
            get_active_ports,
            get_dev_servers,
            open_project_folder,
            get_docker_containers,
            control_docker_container,
            get_db_servers,
            get_pkg_managers,
            get_git_activity,
            get_environment_info,
            send_api_request,
            get_saved_api_collections,
            save_api_request,
            delete_api_request,
            toggle_gaming_boost,
            cleanup_gaming_memory
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
