use serde::Serialize;
use std::sync::Mutex;
use sysinfo::{Components, Disks, Networks, System};
use tauri::State;

#[derive(Serialize)]
struct DiskInfo {
    name: String,
    total_space: u64,
    available_space: u64,
}

#[derive(Serialize)]
struct ProcessInfo {
    name: String,
    pid: u32,
    cpu_usage: f32,
    memory: u64,
}

#[derive(Serialize)]
struct SystemStats {
    cpu_usage: f32,
    cpu_cores: usize,
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
}

pub struct AppState {
    sys: Mutex<System>,
}

#[tauri::command]
fn get_system_stats(state: State<'_, AppState>) -> SystemStats {
    let mut sys = state.sys.lock().unwrap();

    // System metrics
    sys.refresh_cpu_all();
    sys.refresh_memory();

    let cpu_usage = sys.global_cpu_usage();
    let cpu_cores = sys.cpus().len();
    let memory_used = sys.used_memory();
    let memory_total = sys.total_memory();
    let os_name = System::name().unwrap_or_else(|| "Unknown".to_string());
    let os_version = System::os_version().unwrap_or_else(|| "Unknown".to_string());
    let uptime = System::uptime();

    // Disks metrics (separate struct in sysinfo 0.33)
    let disks_info = Disks::new_with_refreshed_list();
    let disks = disks_info
        .iter()
        .map(|disk| DiskInfo {
            name: disk.name().to_string_lossy().into_owned(),
            total_space: disk.total_space(),
            available_space: disk.available_space(),
        })
        .collect();

    // Network metrics (separate struct in sysinfo 0.33)
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
    for c in &components {
        let label = c.label().to_lowercase();
        if label.contains("cpu") || label.contains("package") || label.contains("core") {
            cpu_temp = c.temperature();
            break;
        }
    }

    // Processes metrics
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);

    let mut processes: Vec<ProcessInfo> = sys
        .processes()
        .iter()
        .map(|(pid, process)| ProcessInfo {
            name: process.name().to_string_lossy().into_owned(),
            pid: pid.as_u32(),
            cpu_usage: process.cpu_usage(),
            memory: process.memory(),
        })
        .collect();

    // Sort by CPU usage descending
    processes.sort_by(|a, b| {
        b.cpu_usage
            .partial_cmp(&a.cpu_usage)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    processes.truncate(10);

    SystemStats {
        cpu_usage,
        cpu_cores,
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
    }
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
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, get_system_stats])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
