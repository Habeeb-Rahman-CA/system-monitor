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
struct ProcessInfo {
    name: String,
    pid: u32,
    parent_pid: Option<u32>,
    cpu_usage: f32,
    memory: u64,
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
    vram_used: u64,
    vram_total: u64,
    battery_level: Option<f32>,
    disk_read_speed: u64,
    disk_write_speed: u64,
    ping: u32,
    wifi_signal: u32,
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

    // 5. Collect temp/gpu from CACHE
    let mut cpu_temp = None;
    let mut gpu_name = "N/A".to_string();
    for c in state.components.lock().unwrap().iter() {
        let label = c.label().to_lowercase();
        if cpu_temp.is_none() && (label.contains("cpu") || label.contains("package")) {
            cpu_temp = c.temperature();
        }
        if (label.contains("gpu") || label.contains("nvidia") || label.contains("amd"))
            && gpu_name == "N/A"
        {
            gpu_name = c.label().to_string();
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
        gpu_usage: 0.0,
        vram_used: 0,
        vram_total: 0,
        battery_level: None,
        disk_read_speed,
        disk_write_speed,
        ping: *state.ping.lock().unwrap(),
        wifi_signal: *state.wifi_signal.lock().unwrap(),
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
fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
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
            save_export
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
