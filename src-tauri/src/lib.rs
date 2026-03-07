use serde::Serialize;
use std::sync::Mutex;
use sysinfo::{Components, Disks, Networks, System};
use tauri::State;

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
    last_disk_total_read: Mutex<u64>,
    last_disk_total_write: Mutex<u64>,
    last_sample_time: Mutex<std::time::Instant>,
}

#[tauri::command]
fn get_system_stats(state: State<'_, AppState>) -> SystemStats {
    let mut sys = state.sys.lock().unwrap();

    // System metrics
    sys.refresh_cpu_all();
    sys.refresh_memory();

    let cpu_usage = sys.global_cpu_usage();
    let cpu_cores = sys.cpus().len();
    let physical_cores = sys.physical_core_count().unwrap_or(cpu_cores);
    let cpu_model = sys
        .cpus()
        .get(0)
        .map(|c| c.brand().to_string())
        .unwrap_or_else(|| "Unknown".to_string());
    let cpu_arch = System::cpu_arch();
    let cpu_freq = sys.cpus().get(0).map(|c| c.frequency()).unwrap_or(0);

    let memory_used = sys.used_memory();
    let memory_total = sys.total_memory();
    let os_name = System::name().unwrap_or_else(|| "Unknown".to_string());
    let os_version = System::os_version().unwrap_or_else(|| "Unknown".to_string());
    let uptime = System::uptime();

    // Disks metrics
    let disks_info = Disks::new_with_refreshed_list();
    let disks = disks_info
        .iter()
        .map(|disk| DiskInfo {
            name: disk.name().to_string_lossy().into_owned(),
            total_space: disk.total_space(),
            available_space: disk.available_space(),
            kind: format!("{:?}", disk.kind()),
        })
        .collect();

    // Network metrics
    let networks = Networks::new_with_refreshed_list();
    let mut net_received = 0;
    let mut net_transmitted = 0;
    for (_interface_name, data) in &networks {
        net_received += data.total_received();
        net_transmitted += data.total_transmitted();
    }

    let cpus: Vec<f32> = sys.cpus().iter().map(|cpu| cpu.cpu_usage()).collect();

    let components = Components::new_with_refreshed_list();
    let mut cpu_temp = None;
    let battery_level = None;
    let mut gpu_name = "N/A".to_string();

    for c in &components {
        let label = c.label().to_lowercase();
        if cpu_temp.is_none()
            && (label.contains("cpu") || label.contains("package") || label.contains("core"))
        {
            cpu_temp = c.temperature();
        }
        if label.contains("battery") {
            // Placeholder for battery
        }
        if (label.contains("gpu") || label.contains("nvidia") || label.contains("amd"))
            && gpu_name == "N/A"
        {
            gpu_name = c.label().to_string();
        }
    }

    // Processes metrics
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);

    // Disk Speed Calculation helper (using processes as a proxy for total I/O if global is unavailable)
    let mut current_disk_read_total = 0;
    let mut current_disk_write_total = 0;

    let mut processes: Vec<ProcessInfo> = sys
        .processes()
        .iter()
        .map(|(pid, process)| {
            let disk_usage = process.disk_usage();
            current_disk_read_total += disk_usage.total_read_bytes;
            current_disk_write_total += disk_usage.total_written_bytes;

            ProcessInfo {
                name: process.name().to_string_lossy().into_owned(),
                pid: pid.as_u32(),
                parent_pid: process.parent().map(|p| p.as_u32()),
                cpu_usage: process.cpu_usage(),
                memory: process.memory(),
            }
        })
        .collect();

    // Calculate disk speed
    let now = std::time::Instant::now();
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

    // Advanced Metrics - Ping
    let ping = get_latency();

    // Advanced Metrics - WiFi
    let wifi_signal = get_wifi_signal();

    // Advanced Metrics - GPU (Placeholder for now, will attempt query)
    let (gpu_usage, vram_used, vram_total) = get_gpu_metrics(&gpu_name);

    // Sort processes
    // Don't truncate manually here, let the frontend handle filtering/searching
    processes.sort_by(|a, b| {
        b.cpu_usage
            .partial_cmp(&a.cpu_usage)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    SystemStats {
        cpu_usage,
        cpu_cores,
        physical_cores,
        cpu_model,
        cpu_arch,
        cpu_freq,
        cpus,
        cpu_temp,
        memory_used,
        memory_total,
        os_name,
        os_version,
        uptime,
        disks,
        net_received,
        net_transmitted,
        processes,
        gpu_name,
        gpu_usage,
        vram_used,
        vram_total,
        battery_level,
        disk_read_speed,
        disk_write_speed,
        ping,
        wifi_signal,
    }
}

fn get_latency() -> u32 {
    let now = std::time::Instant::now();
    // Using a quick TCP connect to 8.8.8.8:53 (Google DNS) as a latency measure
    if let Ok(_) = std::net::TcpStream::connect_timeout(
        &"8.8.8.8:53".parse().unwrap(),
        std::time::Duration::from_millis(500),
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
        let output = Command::new("netsh")
            .args(&["wlan", "show", "interfaces"])
            .output();

        if let Ok(out) = output {
            let s = String::from_utf8_lossy(&out.stdout);
            for line in s.lines() {
                if line.contains("Signal") {
                    if let Some(pct_str) = line.split(':').last() {
                        if let Ok(pct) = pct_str.trim().trim_end_matches('%').parse::<u32>() {
                            return pct;
                        }
                    }
                }
            }
        }
    }
    0
}

fn get_gpu_metrics(_name: &str) -> (f32, u64, u64) {
    // This is OS and hardware specific.
    // On Windows, one could use WMI or Performance Counters.
    // For now, let's try a simple fallback.
    (0.0, 0, 0)
}

#[tauri::command]
fn kill_process(state: State<'_, AppState>, pid: u32) -> Result<(), String> {
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
            if let Ok(services) = serde_json::from_str::<serde_json::Value>(&s) {
                if let Some(arr) = services.as_array() {
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
        let output = Command::new("powershell")
            .args(&["-Command", "Get-CimInstance Win32_StartupCommand | Select-Object Name, Command, Location | ConvertTo-Json"])
            .output();

        if let Ok(out) = output {
            let s = String::from_utf8_lossy(&out.stdout);
            if let Ok(apps) = serde_json::from_str::<serde_json::Value>(&s) {
                if let Some(arr) = apps.as_array() {
                    return arr
                        .iter()
                        .map(|v| StartupInfo {
                            name: v["Name"].as_str().unwrap_or("").to_string(),
                            command: v["Command"].as_str().unwrap_or("").to_string(),
                            location: v["Location"].as_str().unwrap_or("").to_string(),
                        })
                        .collect();
                } else if let Some(obj) = apps.as_object() {
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

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState {
            sys: Mutex::new(System::new_all()),
            last_disk_total_read: Mutex::new(0),
            last_disk_total_write: Mutex::new(0),
            last_sample_time: Mutex::new(std::time::Instant::now()),
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            get_system_stats,
            kill_process,
            get_services,
            control_service,
            get_startup_apps
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
