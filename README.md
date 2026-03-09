<p align="center">
  <img src="src-tauri/icons/128x128@2x.png" width="100" alt="ZOH Logo" />
</p>

<h1 align="center">ZOH: AI SYSTEM MONITOR</h1>

<p align="center">
  <strong>A premium, AI-powered system monitoring desktop application with CLI support, real-time analytics, and developer tools.</strong>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/Tauri-2.0-blue?style=for-the-badge&logo=tauri" alt="Tauri 2" />
  <img src="https://img.shields.io/badge/Angular-20-red?style=for-the-badge&logo=angular" alt="Angular 20" />
  <img src="https://img.shields.io/badge/Rust-Backend-orange?style=for-the-badge&logo=rust" alt="Rust" />
  <img src="https://img.shields.io/badge/Platform-Windows-0078D6?style=for-the-badge&logo=windows" alt="Windows" />
  <img src="https://img.shields.io/badge/Version-0.1.0-green?style=for-the-badge" alt="Version" />
</p>

---

## ✨ Overview

**ZOH** is a lightweight, high-performance desktop application that provides deep visibility into your system's hardware and software. Built with **Tauri 2 + Angular 20 + Rust**, it delivers native performance with a stunning glassmorphism UI — complete with an AI-powered advisor that runs **100% offline** on your machine, a full CLI mode, and a developer productivity toolkit.

---

## 🚀 Features

### 🖥️ Splash Screen

- Premium animated launch screen with **ZOH branding**
- Pulsating radar logo with glow effects
- Progress bar animation during system core initialization
- Auto-dismisses once the backend is ready

### 📊 Real-Time Dashboard

| Feature | Description |
|---|---|
| **CPU Monitoring** | Live CPU usage percentage with historical line chart (60s rolling window) and temperature readout |
| **Per-Core Activity** | Individual utilization charts for every logical CPU core |
| **Deep CPU Insights** | Clock speed, logical/physical core breakdown, load averages, and core utilization matrix |
| **Memory Usage** | RAM consumption with used/cached/free breakdown, segment bar, and swap monitoring |
| **GPU Monitoring** | Real-time GPU utilization chart with model name, temperature, clock speed, VRAM, and fan speed |
| **Network Deep Monitor** | Download/upload speeds, local/public IP, ping latency, connection count, and live traffic chart |
| **Disk I/O Activity** | Read/write speed with partition breakdown, capacity bars, and disk health badge |
| **Hardware Sensors** | Live temperature readings from all detected sensors (CPU, GPU, mainboard) with thermal bars |
| **Power & Battery** | Battery percentage, charging status, time remaining, health %, power usage (W), and cycle count |
| **System Health & Uptime** | Dynamic health score (0–100%), system uptime, last boot time, and crash reports count |
| **Resource Hog Finder** | **Top 5 apps slowing your PC** — ranked by composite CPU + RAM impact with quick-kill actions |
| **App Usage Tracking** | Tracks which applications consume the most runtime with progress bars |
| **Environment Card** | CPU model, OS version, and GPU name at a glance |
| **Live Indicator** | Pulsing green dot to confirm metrics are streaming in real-time |

### 📈 Performance Analytics

- **AI System Health Score** — "Your system performance: 92/100 — Excellent" with CPU wellness, thermal stability, and memory efficiency breakdowns
- **CPU Utilization History** — 60-second rolling timeline graph
- **RAM Usage History** — Memory pressure over time
- **Network Traffic History** — Upload/download trends
- **Disk I/O History** — Read/write activity timeline
- **GPU Usage History** — Graphics load over time
- **Ping/Latency History** — Network stability tracking

### ⚙️ Process Controller

- 🔍 **Search & Filter** — Instantly search processes by name or PID
- ⭐ **Favorites** — Pin important processes to the top of the list
- 📊 **Multi-Column Sorting** — Sort by Name, PID, CPU%, or RAM (ascending/descending)
- 🌳 **Tree View** — Visualize parent-child process hierarchy
- ❌ **Kill Process** — Terminate any running process with one click

### 🛠️ System Management

- **Windows Services Manager** — Browse all Windows services, view their status, and start/stop them directly
- **Startup Apps Inspector** — View all startup programs with their launch commands and registry locations
- **Search Services & Startup** — Filter management lists with instant search
- **Custom Dashboard** — Toggle individual widgets on/off to personalize your monitoring layout

### 💻 Developer Control Center

| Feature | Description |
|---|---|
| **Smart Dev Advisor** | Proactive warnings and tips for developers — detects port conflicts, heavy processes, large Git diffs, and Docker cleanup opportunities |
| **Local Dev Server Detection** | Auto-detects running frameworks (React, Vite, Node.js, Python, PHP) with port and status |
| **Active Ports Scanner** | Lists all open ports with associated processes and state |
| **Docker Container Manager** | View and control Docker containers directly from the app |
| **Database Server Monitor** | Detects running database servers (MySQL, PostgreSQL, MongoDB, Redis) |
| **Package Manager Tracker** | Monitors active npm/yarn/pip/cargo installs |
| **Git Activity Dashboard** | Branch info, uncommitted changes, last commit, and repo dirty state |
| **Environment Inspector** | Versions for Node.js, Python, Rust, Git, and shell type |
| **API Testing Tool** | Built-in Postman-like HTTP client (GET, POST, PUT, DELETE) with saved collections |

### 🎮 Gaming Mode

- **Gaming Boost Toggle** — Switches power plan to High Performance
- **Memory Cleanup** — Frees working sets of non-essential background processes

### 📄 Export & Reporting Center

| Report | Format | Details |
|---|---|---|
| **System Diagnostic** | PDF | Comprehensive report with hardware specs, real-time status, and summary metrics |
| **Performance Logs** | CSV | Last 60 seconds of telemetry data (CPU, RAM, Network, Ping) |
| **Hardware Summary** | JSON | Full technical spec of all detected system components |
| **Reliability Report** | PDF | Uptime, boot time, latency, and connection stability metrics |

### 🤖 AI Performance Advisor

A built-in intelligent advisor that analyzes your system metrics and provides actionable recommendations.

#### Rule-Based Engine
- Detects **CPU overload**, **RAM pressure**, **high disk usage**, and **overheating**
- Monitors individual process behavior (e.g., Chrome memory consumption)
- Auto-runs every **10 seconds** with a computed **Health Score (0–100)**
- Advice cards categorized by severity: 🔴 Critical · 🟡 Warning · 🔵 Info · 🟢 Good

#### Local AI (LLM) Mode
- Runs **100% offline** using [WebLLM](https://github.com/mlc-ai/web-llm) with WebGPU acceleration
- Choose from multiple models:
  - **LLaMA 3.2 1B** — Fast (~0.6 GB)
  - **LLaMA 3.2 3B** — Smarter (~1.8 GB)
  - **Phi 3.5 Mini** — Balanced (~2.0 GB)
  - **Gemma 2 2B** — Google (~1.5 GB)
  - **Mistral 7B** — Powerful (~4.0 GB)
- Models download once and run entirely locally — no cloud dependency
- Streaming response preview while the LLM is thinking

### 🖥️ CLI Mode (Command Line Interface)

Run ZOH directly from your terminal without opening the GUI.

```bash
# Quick system scan
./zoh scan

# Real-time CPU monitor (Ctrl+C to stop)
./zoh monitor --cpu

# Clean temporary files
./zoh clean-temp

# Help
./zoh help
```

**Example output:**
```
--- ZOH: AI SYSTEM SCAN (CLI MODE) ---
System Name:    Windows
System Version: 11 (26200)
Host Name:      HabeebRahman
Arch:           x86_64

CPU Model:      12th Gen Intel(R) Core(TM) i3-1215U
Global Usage:   67.55%
Total Memory:   7863 MB
Used Memory:    6785 MB

Uptime:         45h 1m
--------------------------------------
```

### 🎨 UI & Design

- **Splash Screen** — Animated ZOH branding with progress loader on startup
- **Glassmorphism Design** — Translucent panels with backdrop-filter blur and saturation
- **Dark & Light Themes** — One-click toggle with smooth animated transitions
- **Ambient Orb Animations** — Floating gradient orbs for a premium, Apple-inspired aesthetic
- **Collapsible Sidebar** — Animated hamburger-to-X collapse with smooth transitions
- **Custom Titlebar** — Draggable titlebar with minimize, maximize, close, and utility buttons
- **Responsive Grid Layout** — Auto-fill dashboard grid adapts to any window size
- **Fade-In Animations** — Cards and panels gracefully animate into view
- **Slide-In Advisor Panel** — AI advisor slides in from the right with backdrop overlay

### 🪟 Window Controls

- **Mini Mode** — Compact overlay showing essential metrics, perfect for gaming or multitasking
- **Always-on-Top** — Pin the window above all other applications
- **Custom Window Chrome** — Frameless window with native drag region

### 🔔 Smart Alerts

- Automatic threshold-based alerts for critical system events:
  - CPU usage > 90%
  - RAM usage > 90%
  - CPU temperature > 85°C
  - Disk usage > 95%
- Toast-style alert notifications with dismiss buttons
- Categorized as **Warning**, **Critical**, or **Info**

---

## 🏗️ Tech Stack

| Layer | Technology |
|---|---|
| **Frontend** | Angular 20, TypeScript, Chart.js |
| **Backend** | Rust (Tauri 2 commands) |
| **System API** | `sysinfo` crate (CPU, RAM, Disk, Network, Processes, Sensors) |
| **Desktop Shell** | Tauri 2 (transparent window, custom titlebar) |
| **Charts** | Chart.js with real-time streaming updates |
| **PDF Export** | jsPDF + jspdf-autotable |
| **AI/LLM** | @mlc-ai/web-llm (WebGPU, runs offline) |
| **Battery** | starship-battery crate |
| **HTTP Client** | reqwest crate (for API testing tool) |
| **Styling** | Vanilla CSS with CSS variables, glassmorphism |

---

## 📦 Prerequisites

- [Node.js](https://nodejs.org/) (v18+)
- [Rust](https://www.rust-lang.org/tools/install) (latest stable)
- [Tauri CLI](https://tauri.app/start/) (`npm install -g @tauri-apps/cli`)

---

## 🛠️ Getting Started

### 1. Clone the repository

```bash
git clone https://github.com/Habeeb-Rahman-CA/system-monitor.git
cd system-monitor
```

### 2. Install dependencies

```bash
npm install
```

### 3. Run in development mode

```bash
npm run tauri dev
```

### 4. Build for production

```bash
npm run tauri build
```

The installer will be generated in `src-tauri/target/release/bundle/`.

### 5. Use CLI Mode (after build)

```bash
./zoh scan
./zoh monitor --cpu
./zoh clean-temp
```

---

## 📁 Project Structure

```
system-monitor/
├── src/                          # Angular frontend
│   ├── app/
│   │   ├── app.component.ts      # Main application logic
│   │   ├── app.component.html    # Dashboard, Performance, Management, Dev, Gaming, Reports UI
│   │   └── app.component.css     # Glassmorphism styles & design system
│   ├── styles.css                # Global theme (dark/light mode, orbs)
│   ├── index.html                # Entry point
│   └── assets/                   # Static assets
├── src-tauri/                    # Rust backend (Tauri)
│   ├── src/
│   │   ├── lib.rs                # System stats, CLI handler, process/service mgmt, dev tools, API client
│   │   └── main.rs               # Tauri entry point
│   ├── tauri.conf.json           # Window config (frameless, ZOH branding)
│   ├── Cargo.toml                # Rust dependencies
│   └── icons/                    # App icons (ICO, ICNS, PNG)
├── zoh                           # CLI wrapper (bash/MINGW64)
├── zoh.bat                       # CLI wrapper (Windows CMD)
├── package.json                  # Node dependencies & scripts
└── README.md                     # You are here
```

---

## 🧠 Rust Backend Commands

| Command | Description |
|---|---|
| `get_system_stats` | Returns comprehensive system telemetry (CPU, RAM, GPU, disks, processes, network, battery, sensors, ping, WiFi, health score) |
| `kill_process` | Terminates a process by PID |
| `get_services` | Lists all Windows services with status |
| `control_service` | Start or stop a Windows service by name |
| `get_startup_apps` | Reads startup entries from the Windows Registry |
| `get_active_ports` | Lists all open TCP/UDP ports with process info |
| `get_dev_servers` | Detects running local development servers |
| `get_docker_containers` | Lists Docker containers with status and ports |
| `get_db_servers` | Detects running database servers |
| `get_pkg_managers` | Monitors active package manager processes |
| `get_git_activity` | Retrieves Git repository status |
| `get_environment_info` | Returns installed dev tool versions |
| `send_api_request` | Executes HTTP requests (GET/POST/PUT/DELETE) |
| `toggle_gaming_boost` | Switches Windows power plan for gaming |
| `cleanup_gaming_memory` | Frees memory from background processes |

### CLI Commands

| Command | Description |
|---|---|
| `zoh scan` | Quick system health and hardware summary |
| `zoh monitor --cpu` | Real-time terminal CPU usage tracker |
| `zoh clean-temp` | Purge system temporary files |
| `zoh help` | Show available CLI commands |

---

## 📝 License

This project is private and not currently published under an open-source license.

---

<p align="center">
  Built with ❤️ by <strong>Habeeb Rahman</strong> using <strong>Tauri 2</strong> + <strong>Angular 20</strong> + <strong>Rust</strong>
</p>
