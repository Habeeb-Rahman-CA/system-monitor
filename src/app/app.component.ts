import { Component, OnInit, OnDestroy, ViewChild, ElementRef, AfterViewInit, ViewChildren, QueryList } from "@angular/core";
import { CommonModule } from "@angular/common";
import { invoke } from "@tauri-apps/api/core";
import { Chart, registerables } from 'chart.js';

Chart.register(...registerables);

interface DiskInfo {
  name: string;
  total_space: number;
  available_space: number;
}

interface ProcessInfo {
  name: string;
  pid: number;
  cpu_usage: number;
  memory: number;
}

interface SystemStats {
  cpu_usage: number;
  cpu_cores: number;
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

  systemStats: SystemStats | null = null;
  interval: any;

  cpuChart: Chart | null = null;
  memoryChart: Chart | null = null;
  coreCharts: Chart[] = [];

  cpuHistory: number[] = new Array(30).fill(0);
  memoryHistory: number[] = new Array(30).fill(0);
  coreHistories: number[][] = [];

  lastNetReceived = 0;
  lastNetTransmitted = 0;
  netSpeedIn = 0; // Bytes per second
  netSpeedOut = 0; // Bytes per second

  ngOnInit() {
    this.interval = setInterval(() => {
      this.fetchStats();
    }, 1000);
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

      this.cpuHistory.push(stats.cpu_usage);
      this.cpuHistory.shift();
      const memPct = (stats.memory_used / stats.memory_total) * 100;
      this.memoryHistory.push(memPct);
      this.memoryHistory.shift();

      if (this.coreHistories.length === 0) {
        this.coreHistories = stats.cpus.map(() => new Array(30).fill(0));
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
          labels: new Array(30).fill(''),
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
}
