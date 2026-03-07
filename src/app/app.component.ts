import { Component, OnInit, OnDestroy, ViewChild, ElementRef, AfterViewInit, ViewChildren, QueryList } from "@angular/core";
import { CommonModule } from "@angular/common";
import { invoke } from "@tauri-apps/api/core";
import { Chart, registerables } from 'chart.js';
import { getCurrentWindow, PhysicalSize } from "@tauri-apps/api/window";
import { jsPDF } from 'jspdf';
import autoTable from 'jspdf-autotable';

const appWindow = getCurrentWindow();

Chart.register(...registerables);

interface Alert {
  id: string;
  type: 'warning' | 'critical' | 'info';
  title: string;
  message: string;
  timestamp: number;
}

interface DiskInfo {
  name: string;
  total_space: number;
  available_space: number;
  kind: string;
}

interface ServiceInfo {
  name: string;
  display_name: string;
  status: string;
}

interface StartupInfo {
  name: string;
  command: string;
  location: string;
}

interface ProcessInfo {
  name: string;
  pid: number;
  parent_pid: number | null;
  cpu_usage: number;
  memory: number;
  isFavorite?: boolean;
  isExpanded?: boolean;
}

type ProcessSortKey = 'name' | 'cpu_usage' | 'memory' | 'pid';

interface SystemStats {
  cpu_usage: number;
  cpu_cores: number;
  physical_cores: number;
  cpu_model: string;
  cpu_arch: string;
  cpu_freq: number;
  cpus: number[];
  cpu_temp: number | null;
  memory_used: number;
  memory_total: number;
  os_name: string;
  os_version: string;
  uptime: number;
  disks: DiskInfo[];
  net_received: number;
  net_transmitted: number;
  processes: ProcessInfo[];
  gpu_name: string;
  gpu_usage: number;
  vram_used: number;
  vram_total: number;
  battery_level: number | null;
  disk_read_speed: number;
  disk_write_speed: number;
  ping: number;
  wifi_signal: number;
}

@Component({
  selector: "app-root",
  standalone: true,
  imports: [CommonModule],
  templateUrl: "./app.component.html",
  styleUrl: "./app.component.css",
})
export class AppComponent implements OnInit, OnDestroy, AfterViewInit {
  @ViewChild('cpuCanvas') cpuCanvas!: ElementRef<HTMLCanvasElement>;
  @ViewChild('memoryCanvas') memoryCanvas!: ElementRef<HTMLCanvasElement>;
  @ViewChildren('coreCanvas') coreCanvases!: QueryList<ElementRef<HTMLCanvasElement>>;
  @ViewChild('gpuCanvas') gpuCanvas!: ElementRef<HTMLCanvasElement>;
  @ViewChild('pingCanvas') pingCanvas!: ElementRef<HTMLCanvasElement>;
  @ViewChild('netTrafficCanvas') netTrafficCanvas!: ElementRef<HTMLCanvasElement>;
  @ViewChild('diskIOHistoryCanvas') diskIOHistoryCanvas!: ElementRef<HTMLCanvasElement>;

  systemStats: SystemStats | null = null;
  interval: any;

  cpuChart: Chart | null = null;
  memoryChart: Chart | null = null;
  gpuChart: Chart | null = null;
  pingChart: Chart | null = null;
  netTrafficChart: Chart | null = null;
  diskIOChart: Chart | null = null;
  coreCharts: Chart[] = [];

  readonly HISTORY_LIMIT = 60; // 1 minute of history
  cpuHistory: number[] = new Array(60).fill(0);
  memoryHistory: number[] = new Array(60).fill(0);
  gpuHistory: number[] = new Array(60).fill(0);
  pingHistory: number[] = new Array(60).fill(0);
  netDownHistory: number[] = new Array(60).fill(0);
  netUpHistory: number[] = new Array(60).fill(0);
  diskReadHistory: number[] = new Array(60).fill(0);
  diskWriteHistory: number[] = new Array(60).fill(0);
  coreHistories: number[][] = [];

  // App Usage Tracking
  appUsageMap: Map<string, number> = new Map(); // name -> seconds
  topAppsUsage: { name: string, time: number }[] = [];

  lastNetReceived = 0;
  lastNetTransmitted = 0;
  netSpeedIn = 0; // Bytes per second
  netSpeedOut = 0; // Bytes per second

  // Process Management State
  searchTerm = '';
  sortKey: ProcessSortKey = 'cpu_usage';
  sortDir: 'asc' | 'desc' = 'desc';
  favorites: Set<number> = new Set();
  expandedParents: Set<number> = new Set();
  showTree = false;

  // Management State
  activeTab: 'dashboard' | 'management' | 'reports' = 'dashboard';
  services: ServiceInfo[] = [];
  startupApps: StartupInfo[] = [];
  managementSearchTerm = '';
  mgmtSubTab: 'services' | 'startup' = 'services';
  isLoadingMgmt = false;
  lastMgmtRefresh = 0;

  // Cached Filtered Lists
  p_filteredProcesses: ProcessInfo[] = [];
  p_filteredServices: ServiceInfo[] = [];
  p_filteredStartupApps: StartupInfo[] = [];

  // Alerts State
  alerts: Alert[] = [];
  alertThresholds = {
    cpu: 90,
    ram: 90,
    temp: 85,
    disk: 95
  };

  // Sidebar & Theme State
  isDarkMode = true;
  isMiniMode = false;
  isAlwaysOnTop = false;
  isSidebarCollapsed = false;

  // Customizable Dashboard (Widget Visibility)
  widgetVisibility = {
    cpu: true,
    cores: true,
    ram: true,
    gpu: true,
    net: true,
    disk: true,
    conn: true,
    usage: true,
    info: true,
    processes: true
  };

  ngOnInit() {
    this.interval = setInterval(() => {
      this.fetchStats();
      const now = Date.now();
      if (this.activeTab === 'management' && !this.isLoadingMgmt && (now - this.lastMgmtRefresh > 5000)) {
        this.refreshManagementData();
      }
    }, 1000);
  }

  updateCaches() {
    this.updateFilteredProcesses();
    this.updateFilteredServices();
    this.updateFilteredStartupApps();
  }

  updateFilteredProcesses() {
    if (!this.systemStats) return;

    let list = this.systemStats.processes.map(p => ({
      ...p,
      isFavorite: this.favorites.has(p.pid)
    }));

    // Search filter
    if (this.searchTerm) {
      const term = this.searchTerm.toLowerCase();
      list = list.filter(p => p.name.toLowerCase().includes(term) || p.pid.toString().includes(term));
    }

    // Sort
    list.sort((a, b) => {
      if (a.isFavorite && !b.isFavorite) return -1;
      if (!a.isFavorite && b.isFavorite) return 1;

      let valA = a[this.sortKey] as any;
      let valB = b[this.sortKey] as any;

      if (typeof valA === 'string') {
        valA = valA.toLowerCase();
        valB = (valB as string).toLowerCase();
      }

      if (valA < valB) return this.sortDir === 'asc' ? -1 : 1;
      if (valA > valB) return this.sortDir === 'asc' ? 1 : -1;
      return 0;
    });

    if (this.showTree && !this.searchTerm) {
      this.p_filteredProcesses = this.buildProcessTree(list);
    } else {
      this.p_filteredProcesses = list.slice(0, 50);
    }
  }

  updateFilteredServices() {
    if (!this.managementSearchTerm) {
      this.p_filteredServices = this.services;
    } else {
      const term = this.managementSearchTerm.toLowerCase();
      this.p_filteredServices = this.services.filter(s =>
        s.name.toLowerCase().includes(term) ||
        s.display_name.toLowerCase().includes(term)
      );
    }
  }

  updateFilteredStartupApps() {
    if (!this.managementSearchTerm) {
      this.p_filteredStartupApps = this.startupApps;
    } else {
      const term = this.managementSearchTerm.toLowerCase();
      this.p_filteredStartupApps = this.startupApps.filter(a =>
        a.name.toLowerCase().includes(term) ||
        a.command.toLowerCase().includes(term)
      );
    }
  }

  ngAfterViewInit() {
    this.initMainCharts();
  }

  ngOnDestroy() {
    if (this.interval) {
      clearInterval(this.interval);
    }
  }

  async fetchStats() {
    try {
      const stats = await invoke<SystemStats>("get_system_stats");

      const firstLoad = !this.systemStats;

      if (this.lastNetReceived > 0) {
        this.netSpeedIn = stats.net_received - this.lastNetReceived;
        this.netSpeedOut = stats.net_transmitted - this.lastNetTransmitted;
      }
      this.lastNetReceived = stats.net_received;
      this.lastNetTransmitted = stats.net_transmitted;

      this.systemStats = stats;
      this.updateCaches();
      this.checkAlerts(stats);
      this.cpuHistory.push(stats.cpu_usage);
      this.cpuHistory.shift();
      const memPct = (stats.memory_used / stats.memory_total) * 100;
      this.memoryHistory.push(memPct);
      this.memoryHistory.shift();

      this.gpuHistory.push(stats.gpu_usage || 0);
      this.gpuHistory.shift();

      this.pingHistory.push(stats.ping || 0);
      this.pingHistory.shift();

      this.netDownHistory.push(this.netSpeedIn);
      this.netDownHistory.shift();
      this.netUpHistory.push(this.netSpeedOut);
      this.netUpHistory.shift();

      this.diskReadHistory.push(stats.disk_read_speed);
      this.diskReadHistory.shift();
      this.diskWriteHistory.push(stats.disk_write_speed);
      this.diskWriteHistory.shift();

      // Track App Usage (Top processes in this sample)
      stats.processes.slice(0, 10).forEach(p => {
        const current = this.appUsageMap.get(p.name) || 0;
        this.appUsageMap.set(p.name, current + 1);
      });
      this.updateAppUsageList();

      if (this.coreHistories.length === 0) {
        this.coreHistories = stats.cpus.map(() => new Array(this.HISTORY_LIMIT).fill(0));
      }
      stats.cpus.forEach((usage, i) => {
        this.coreHistories[i].push(usage);
        this.coreHistories[i].shift();
      });

      if (firstLoad) {
        setTimeout(() => this.initCoreCharts(), 100);
      }

      this.updateCharts();
    } catch (e) {
      console.error("Failed to fetch system stats", e);
    }
  }

  initMainCharts() {
    const commonOptions = {
      responsive: true,
      maintainAspectRatio: false,
      scales: {
        x: { display: false },
        y: {
          beginAtZero: true,
          max: 100,
          grid: { color: 'rgba(255, 255, 255, 0.05)' },
          ticks: { color: '#888', font: { size: 10 } }
        }
      },
      plugins: { legend: { display: false } },
      elements: { point: { radius: 0 }, line: { tension: 0.4 } },
      animation: { duration: 0 } as any
    };

    if (this.cpuCanvas) {
      this.cpuChart = new Chart(this.cpuCanvas.nativeElement, {
        type: 'line',
        data: {
          labels: new Array(this.HISTORY_LIMIT).fill(''),
          datasets: [{
            data: this.cpuHistory,
            borderColor: '#4facfe',
            borderWidth: 2,
            fill: true,
            backgroundColor: 'rgba(79, 172, 254, 0.1)',
          }]
        },
        options: commonOptions
      });
    }

    if (this.memoryCanvas) {
      this.memoryChart = new Chart(this.memoryCanvas.nativeElement, {
        type: 'line',
        data: {
          labels: new Array(30).fill(''),
          datasets: [{
            data: this.memoryHistory,
            borderColor: '#00f2fe',
            borderWidth: 2,
            fill: true,
            backgroundColor: 'rgba(0, 242, 254, 0.1)',
          }]
        },
        options: commonOptions
      });
    }

    if (this.gpuCanvas) {
      this.gpuChart = new Chart(this.gpuCanvas.nativeElement, {
        type: 'line',
        data: {
          labels: new Array(30).fill(''),
          datasets: [{
            data: this.gpuHistory,
            borderColor: '#ff5e5e',
            borderWidth: 2,
            fill: true,
            backgroundColor: 'rgba(255, 94, 94, 0.1)',
          }]
        },
        options: commonOptions
      });
    }

    if (this.pingCanvas) {
      this.pingChart = new Chart(this.pingCanvas.nativeElement, {
        type: 'line',
        data: {
          labels: new Array(this.HISTORY_LIMIT).fill(''),
          datasets: [{
            data: this.pingHistory,
            borderColor: '#39ff14',
            borderWidth: 2,
            fill: true,
            backgroundColor: 'rgba(57, 255, 20, 0.1)',
          }]
        },
        options: {
          ...commonOptions,
          scales: { ...commonOptions.scales, y: { ...commonOptions.scales.y, max: undefined } }
        }
      });
    }

    if (this.netTrafficCanvas) {
      this.netTrafficChart = new Chart(this.netTrafficCanvas.nativeElement, {
        type: 'line',
        data: {
          labels: new Array(this.HISTORY_LIMIT).fill(''),
          datasets: [
            {
              label: 'Down',
              data: this.netDownHistory,
              borderColor: '#00f2fe',
              borderWidth: 1.5,
              fill: false,
            },
            {
              label: 'Up',
              data: this.netUpHistory,
              borderColor: '#4facfe',
              borderWidth: 1.5,
              fill: false,
            }
          ]
        },
        options: { ...commonOptions, scales: { ...commonOptions.scales, y: { ...commonOptions.scales.y, max: undefined } } }
      });
    }

    if (this.diskIOHistoryCanvas) {
      this.diskIOChart = new Chart(this.diskIOHistoryCanvas.nativeElement, {
        type: 'line',
        data: {
          labels: new Array(this.HISTORY_LIMIT).fill(''),
          datasets: [
            {
              label: 'Read',
              data: this.diskReadHistory,
              borderColor: '#ff5e5e',
              borderWidth: 1.5,
              fill: false,
            },
            {
              label: 'Write',
              data: this.diskWriteHistory,
              borderColor: '#ffcc00',
              borderWidth: 1.5,
              fill: false,
            }
          ]
        },
        options: { ...commonOptions, scales: { ...commonOptions.scales, y: { ...commonOptions.scales.y, max: undefined } } }
      });
    }
  }

  initCoreCharts() {
    const coreOptions = {
      responsive: true,
      maintainAspectRatio: false,
      scales: {
        x: { display: false },
        y: { beginAtZero: true, max: 100, display: false }
      },
      plugins: { legend: { display: false } },
      elements: { point: { radius: 0 }, line: { tension: 0.4 } },
      animation: { duration: 0 } as any
    };

    this.coreCanvases.forEach((canvas, i) => {
      this.coreCharts[i] = new Chart(canvas.nativeElement, {
        type: 'line',
        data: {
          labels: new Array(30).fill(''),
          datasets: [{
            data: this.coreHistories[i],
            borderColor: '#4facfe',
            borderWidth: 1.5,
            fill: true,
            backgroundColor: 'rgba(79, 172, 254, 0.05)',
          }]
        },
        options: coreOptions
      });
    });
  }

  updateCharts() {
    if (this.cpuChart) {
      this.cpuChart.data.datasets[0].data = [...this.cpuHistory];
      this.cpuChart.update();
    }
    if (this.memoryChart) {
      this.memoryChart.data.datasets[0].data = [...this.memoryHistory];
      this.memoryChart.update();
    }
    if (this.gpuChart) {
      this.gpuChart.data.datasets[0].data = [...this.gpuHistory];
      this.gpuChart.update();
    }
    if (this.pingChart) {
      this.pingChart.data.datasets[0].data = [...this.pingHistory];
      this.pingChart.update();
    }
    if (this.netTrafficChart) {
      this.netTrafficChart.data.datasets[0].data = [...this.netDownHistory];
      this.netTrafficChart.data.datasets[1].data = [...this.netUpHistory];
      this.netTrafficChart.update();
    }
    if (this.diskIOChart) {
      this.diskIOChart.data.datasets[0].data = [...this.diskReadHistory];
      this.diskIOChart.data.datasets[1].data = [...this.diskWriteHistory];
      this.diskIOChart.update();
    }
    this.coreCharts.forEach((chart, i) => {
      if (chart) {
        chart.data.datasets[0].data = [...this.coreHistories[i]];
        chart.update();
      }
    });
  }

  formatUptime(seconds: number): string {
    const days = Math.floor(seconds / (24 * 3600));
    const hours = Math.floor((seconds % (24 * 3600)) / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    const s = seconds % 60;

    const parts = [];
    if (days > 0) parts.push(`${days}d`);
    if (hours > 0) parts.push(`${hours}h`);
    if (minutes > 0) parts.push(`${minutes}m`);
    parts.push(`${s}s`);
    return parts.join(' ');
  }

  getDiskUsagePercentage(disk: DiskInfo): number {
    return ((disk.total_space - disk.available_space) / disk.total_space) * 100;
  }

  getMemoryPercentage(): number {
    if (!this.systemStats) return 0;
    return (this.systemStats.memory_used / this.systemStats.memory_total) * 100;
  }

  formatBytes(bytes: number): string {
    if (bytes === 0) return "0 Bytes";
    const k = 1024;
    const sizes = ["Bytes", "KB", "MB", "GB", "TB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + " " + sizes[i];
  }

  // --- Process Management Methods ---

  toggleFavorite(pid: number) {
    if (this.favorites.has(pid)) this.favorites.delete(pid);
    else this.favorites.add(pid);
    this.updateFilteredProcesses();
  }

  async killProcess(pid: number) {
    if (confirm(`Kill process ${pid}?`)) {
      try {
        await invoke('kill_process', { pid });
        this.fetchStats();
      } catch (e) {
        alert("Failed to kill process: " + e);
      }
    }
  }

  setSort(key: ProcessSortKey) {
    if (this.sortKey === key) {
      this.sortDir = this.sortDir === 'asc' ? 'desc' : 'asc';
    } else {
      this.sortKey = key;
      this.sortDir = 'desc';
    }
    this.updateFilteredProcesses();
  }

  onSearch(event: any) {
    this.searchTerm = event.target.value;
    this.updateFilteredProcesses();
  }

  toggleTree() {
    this.showTree = !this.showTree;
    this.updateFilteredProcesses();
  }

  toggleExpand(pid: number) {
    if (this.expandedParents.has(pid)) this.expandedParents.delete(pid);
    else this.expandedParents.add(pid);
  }

  // --- Management Methods ---

  setActiveTab(tab: 'dashboard' | 'management' | 'reports') {
    this.activeTab = tab;
    if (tab === 'management') {
      this.refreshManagementData();
    }
  }

  setMgmtSubTab(subTab: 'services' | 'startup') {
    this.mgmtSubTab = subTab;
  }

  // --- Reports & Exports ---

  exportSystemReport() {
    if (!this.systemStats) return;
    const doc = new jsPDF();
    const stats = this.systemStats;

    doc.setFontSize(22);
    doc.text("System Diagnostic Report", 20, 20);
    doc.setFontSize(10);
    doc.text(`Generated on: ${new Date().toLocaleString()}`, 20, 30);

    doc.setFontSize(14);
    doc.text("1. Hardware Specifications", 20, 45);
    doc.setFontSize(10);
    doc.text(`CPU: ${stats.cpu_model}`, 25, 55);
    doc.text(`GPU: ${stats.gpu_name}`, 25, 60);
    doc.text(`Memory Total: ${this.formatBytes(stats.memory_total)}`, 25, 65);
    doc.text(`OS: ${stats.os_name} (${stats.os_version})`, 25, 70);

    doc.setFontSize(14);
    doc.text("2. Real-time Status", 20, 85);
    doc.setFontSize(10);
    doc.text(`Current CPU Usage: ${stats.cpu_usage.toFixed(1)}%`, 25, 95);
    doc.text(`Memory Used: ${this.formatBytes(stats.memory_used)} (${((stats.memory_used / stats.memory_total) * 100).toFixed(1)}%)`, 25, 100);
    doc.text(`System Uptime: ${this.formatUptime(stats.uptime)}`, 25, 105);

    autoTable(doc, {
      startY: 120,
      head: [['Metric', 'Value']],
      body: [
        ['Process Count', stats.processes.length.toString()],
        ['Active Disks', stats.disks.length.toString()],
        ['Network Download', `${this.formatBytes(this.netSpeedIn)}/s`],
        ['Network Upload', `${this.formatBytes(this.netSpeedOut)}/s`],
        ['Avg Latency', `${stats.ping} ms`]
      ],
      theme: 'grid',
      headStyles: { fillColor: [79, 172, 254], textColor: [255, 255, 255] }
    });

    doc.save(`system_report_${Date.now()}.pdf`);
  }

  exportPerformanceCSV() {
    if (!this.systemStats) return;
    let csv = "Timestamp,CPU_Usage,RAM_Used_Bytes,RAM_Total_Bytes,Net_In_Bps,Net_Out_Bps,Ping_ms\n";

    // Using current history arrays (last 60 seconds)
    for (let i = 0; i < this.HISTORY_LIMIT; i++) {
      const timestamp = new Date(Date.now() - (this.HISTORY_LIMIT - i) * 1000).toISOString();
      const row = [
        timestamp,
        this.cpuHistory[i] || 0,
        this.memoryHistory[i] || 0,
        this.systemStats.memory_total,
        this.netDownHistory[i] || 0,
        this.netUpHistory[i] || 0,
        this.pingHistory[i] || 0
      ].join(",");
      csv += row + "\n";
    }

    const blob = new Blob([csv], { type: 'text/csv' });
    const url = window.URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `perf_logs_${Date.now()}.csv`;
    a.click();
  }

  exportHardwareSummary() {
    if (!this.systemStats) return;
    const report = {
      product: "System Monitor Pro - Hardware Summary",
      timestamp: new Date().toISOString(),
      cpu: {
        model: this.systemStats.cpu_model,
        cores: this.systemStats.cpu_cores,
        freq: this.systemStats.cpu_freq,
        arch: this.systemStats.cpu_arch
      },
      memory: {
        total: this.systemStats.memory_total,
        total_formatted: this.formatBytes(this.systemStats.memory_total)
      },
      gpu: {
        name: this.systemStats.gpu_name
      },
      os: {
        name: this.systemStats.os_name,
        version: this.systemStats.os_version
      },
      disks: this.systemStats.disks
    };

    const blob = new Blob([JSON.stringify(report, null, 2)], { type: 'application/json' });
    const url = window.URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `hardware_summary_${Date.now()}.json`;
    a.click();
  }

  exportUptimeReport() {
    if (!this.systemStats) return;
    const doc = new jsPDF();
    doc.setFontSize(22);
    doc.text("Reliability & Uptime Report", 20, 20);

    doc.setFontSize(12);
    doc.text(`Official Session Report for: ${this.systemStats.os_name}`, 20, 35);

    autoTable(doc, {
      startY: 50,
      head: [['Event', 'Details']],
      body: [
        ['System Boot Time', new Date(Date.now() - this.systemStats.uptime * 1000).toLocaleString()],
        ['Current Session Length', this.formatUptime(this.systemStats.uptime)],
        ['Monitor Session Start', new Date(this.lastNetReceived > 0 ? Date.now() : Date.now()).toLocaleString()], // Placeholder
        ['Avg Response Latency', `${this.systemStats.ping} ms`],
        ['Status', 'HEALTHY']
      ],
      theme: 'striped'
    });

    doc.save(`uptime_report_${Date.now()}.pdf`);
  }

  async refreshManagementData() {
    if (this.isLoadingMgmt) return;
    this.isLoadingMgmt = true;
    try {
      if (this.mgmtSubTab === 'services') {
        this.services = await invoke<ServiceInfo[]>('get_services');
      } else {
        this.startupApps = await invoke<StartupInfo[]>('get_startup_apps');
      }
      this.lastMgmtRefresh = Date.now();
      this.updateFilteredServices();
      this.updateFilteredStartupApps();
    } catch (e) {
      console.error("Failed to fetch management data", e);
    } finally {
      this.isLoadingMgmt = false;
    }
  }

  get filteredServices() {
    return this.p_filteredServices;
  }

  get filteredStartupApps() {
    return this.p_filteredStartupApps;
  }

  get filteredProcesses() {
    return this.p_filteredProcesses;
  }

  // --- Alert System ---

  checkAlerts(stats: SystemStats) {
    const newAlerts: Alert[] = [];

    // 1. CPU Warning
    if (stats.cpu_usage > this.alertThresholds.cpu) {
      newAlerts.push({
        id: 'cpu_high',
        type: 'critical',
        title: 'High CPU Usage',
        message: `CPU is at ${stats.cpu_usage.toFixed(1)}%`,
        timestamp: Date.now()
      });
    }

    // 2. RAM Warning
    const ramPct = (stats.memory_used / stats.memory_total) * 100;
    if (ramPct > this.alertThresholds.ram) {
      newAlerts.push({
        id: 'ram_high',
        type: 'critical',
        title: 'Memory Almost Full',
        message: `RAM usage is at ${ramPct.toFixed(1)}%`,
        timestamp: Date.now()
      });
    }

    // 3. Overheating
    if (stats.cpu_temp && stats.cpu_temp > this.alertThresholds.temp) {
      newAlerts.push({
        id: 'temp_high',
        type: 'critical',
        title: 'Overheating Warning',
        message: `CPU Temperature is dangerous: ${stats.cpu_temp.toFixed(1)}°C`,
        timestamp: Date.now()
      });
    }

    // 4. Disk Space
    stats.disks.forEach(disk => {
      const used = (disk.total_space - disk.available_space) / disk.total_space * 100;
      if (used > this.alertThresholds.disk) {
        newAlerts.push({
          id: `disk_full_${disk.name}`,
          type: 'warning',
          title: 'Disk Almost Full',
          message: `${disk.name} has only ${(disk.available_space / (1024 * 1024 * 1024)).toFixed(1)} GB left`,
          timestamp: Date.now()
        });
      }
    });

    // 5. Internet Connection
    if (stats.ping === null || stats.ping === 0) {
      newAlerts.push({
        id: 'internet_lost',
        type: 'critical',
        title: 'Connection Lost',
        message: 'Unable to reach Google DNS (check network)',
        timestamp: Date.now()
      });
    }

    // Stable update: Keep existing alerts if condition persists, remove if solved
    const activeIds = new Set(newAlerts.map(a => a.id));

    // 1. Remove old alerts that are no longer active
    this.alerts = this.alerts.filter(a => activeIds.has(a.id));

    // 2. Add new alerts ONLY if they are not already in the list
    newAlerts.forEach(newA => {
      if (!this.alerts.some(a => a.id === newA.id)) {
        this.alerts = [newA, ...this.alerts].slice(0, 5);
      }
    });
  }

  dismissAlert(id: string) {
    this.alerts = this.alerts.filter(a => a.id !== id);
  }

  async toggleService(service: ServiceInfo) {
    const action = service.status === 'Running' ? 'stop' : 'start';
    try {
      await invoke('control_service', { name: service.name, action });
      // Powerhell RunAs is async in terms of the window opening, 
      // so we just wait a bit and refresh
      setTimeout(() => this.refreshManagementData(), 1000);
    } catch (e) {
      alert("Failed to control service: " + e);
    }
  }

  onMgmtSearch(event: any) {
    this.managementSearchTerm = event.target.value;
    this.updateFilteredServices();
    this.updateFilteredStartupApps();
  }

  // --- UI/UX Power Features ---

  toggleTheme() {
    this.isDarkMode = !this.isDarkMode;
    const body = document.documentElement;
    if (this.isDarkMode) {
      body.classList.remove('light-mode');
    } else {
      body.classList.add('light-mode');
    }
  }

  toggleSidebar() {
    this.isSidebarCollapsed = !this.isSidebarCollapsed;
  }

  async toggleAlwaysOnTop() {
    this.isAlwaysOnTop = !this.isAlwaysOnTop;
    await appWindow.setAlwaysOnTop(this.isAlwaysOnTop);
  }

  async toggleMiniMode() {
    this.isMiniMode = !this.isMiniMode;
    if (this.isMiniMode) {
      // Switch to a small transparent overlay size
      await appWindow.setSize(new PhysicalSize(300, 400));
      await appWindow.setResizable(false);
      this.activeTab = 'dashboard';
    } else {
      await appWindow.setSize(new PhysicalSize(1000, 800));
      await appWindow.setResizable(true);
    }
  }

  toggleWidget(widget: keyof typeof this.widgetVisibility) {
    this.widgetVisibility[widget] = !this.widgetVisibility[widget];
  }

  private buildProcessTree(list: ProcessInfo[]): ProcessInfo[] {
    const tree: ProcessInfo[] = [];
    const map = new Map<number, ProcessInfo[]>();

    list.forEach(p => {
      const parentId = p.parent_pid || 0;
      if (!map.has(parentId)) map.set(parentId, []);
      map.get(parentId)!.push(p);
    });

    const addChildren = (parentId: number, depth: number) => {
      const children = map.get(parentId);
      if (children) {
        children.forEach(child => {
          tree.push({ ...child, name: ' '.repeat(depth * 3) + (depth > 0 ? '┗ ' : '') + child.name });
          if (this.expandedParents.has(child.pid)) {
            addChildren(child.pid, depth + 1);
          }
        });
      }
    };

    addChildren(0, 0); // Start from roots
    return tree.slice(0, 50);
  }

  updateAppUsageList() {
    this.topAppsUsage = Array.from(this.appUsageMap.entries())
      .map(([name, time]) => ({ name, time }))
      .sort((a, b) => b.time - a.time)
      .slice(0, 5);
  }

  formatDuration(seconds: number): string {
    if (seconds < 60) return `${seconds}s`;
    const mins = Math.floor(seconds / 60);
    const secs = seconds % 60;
    return `${mins}m ${secs}s`;
  }

  async minimizeWindow() {
    await appWindow.minimize();
  }

  async toggleMaximize() {
    await appWindow.toggleMaximize();
  }

  async closeWindow() {
    await appWindow.close();
  }
}
