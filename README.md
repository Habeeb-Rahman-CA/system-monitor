<p align="center">
  <img src="src-tauri/icons/128x128@2x.png" width="100" alt="System Monitor Pro Logo" />
</p>

<h1 align="center">System Monitor Pro</h1>

<p align="center">
  <strong>A premium, real-time system monitoring desktop application with AI-powered performance insights.</strong>
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

**System Monitor Pro** is a lightweight, high-performance desktop application that provides deep visibility into your system's hardware and software. Built with **Tauri 2 + Angular 20 + Rust**, it delivers native performance with a stunning glassmorphism UI — complete with an AI-powered advisor that runs **100% offline** on your machine.

---

## 🚀 Features

### 📊 Real-Time Dashboard

| Feature | Description |
|---|---|
| **CPU Monitoring** | Live CPU usage percentage with historical line chart (60s rolling window) and temperature readout |
| **Per-Core Activity** | Individual utilization charts for every logical CPU core |
| **Memory Usage** | RAM consumption with used/total breakdown and historical trend graph |
| **GPU Monitoring** | Real-time GPU utilization chart with model name display |
| **Network Traffic** | Download/upload speed graphs with live bytes-per-second readout |
| **Disk I/O Activity** | Read/write speed history chart with live throughput display |
| **Connectivity Panel** | Network ping latency graph and WiFi signal strength indicator |
| **System Info Card** | CPU model, OS version, and GPU name at a glance |
| **App Usage Tracking** | Tracks which applications consume the most runtime with progress bars |
| **System Uptime** | Live formatted uptime display (days, hours, minutes, seconds) |
| **Live Indicator** | Pulsing green dot to confirm metrics are streaming in real-time |

### ⚙️ Process Controller

- 🔍 **Search & Filter** — Instantly search processes by name or PID
- ⭐ **Favorites** — Pin important processes to the top of the list
- 📊 **Multi-Column Sorting** — Sort by Name, PID, CPU%, or RAM (ascending/descending)
- 🌳 **Tree View** — Visualize parent-child process hierarchy
- ❌ **Kill Process** — Terminate any running process with one click

### 🛠️ System Management

- **Windows Services Manager** — Browse all Windows services, view their status, and start/stop them directly from the app
- **Startup Apps Inspector** — View all startup programs with their launch commands and registry locations
- **Search Services & Startup** — Filter management lists with instant search
- **Custom Dashboard** — Toggle individual widgets on/off to personalize your monitoring layout

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

### 🎨 UI & Design

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
| **System API** | `sysinfo` crate (CPU, RAM, Disk, Network, Processes) |
| **Desktop Shell** | Tauri 2 (transparent window, custom titlebar) |
| **Charts** | Chart.js with real-time streaming updates |
| **PDF Export** | jsPDF + jspdf-autotable |
| **AI/LLM** | @mlc-ai/web-llm (WebGPU, runs offline) |
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

---

## 📁 Project Structure

```
system-monitor/
├── src/                          # Angular frontend
│   ├── app/
│   │   ├── app.component.ts      # Main application logic (~1400 lines)
│   │   ├── app.component.html    # Dashboard, Management, Reports UI
│   │   └── app.component.css     # Glassmorphism styles (~2000 lines)
│   ├── styles.css                # Global theme (dark/light mode, orbs)
│   ├── index.html                # Entry point
│   └── assets/                   # Static assets
├── src-tauri/                    # Rust backend (Tauri)
│   ├── src/
│   │   ├── lib.rs                # System stats, process kill, services, startup apps
│   │   └── main.rs               # Tauri entry point
│   ├── tauri.conf.json           # Window config (transparent, frameless)
│   ├── Cargo.toml                # Rust dependencies
│   └── icons/                    # App icons (ICO, ICNS, PNG)
├── package.json                  # Node dependencies & scripts
└── README.md                     # You are here
```

---

## 🧠 Rust Backend Commands

| Command | Description |
|---|---|
| `get_system_stats` | Returns comprehensive system telemetry (CPU, RAM, GPU, disks, processes, network, battery, ping, WiFi) |
| `kill_process` | Terminates a process by PID |
| `get_services` | Lists all Windows services with status |
| `control_service` | Start or stop a Windows service by name |
| `get_startup_apps` | Reads startup entries from the Windows Registry |



---

## 📝 License

This project is private and not currently published under an open-source license.

---

<p align="center">
  Built with ❤️ using <strong>Tauri 2</strong> + <strong>Angular 20</strong> + <strong>Rust</strong>
</p>
