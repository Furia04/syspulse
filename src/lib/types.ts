export interface CoreUsage {
  id: number;
  usage: number;
  frequency: number;
}

export interface CpuMetrics {
  global_usage: number;
  core_count: number;
  brand: string;
  cores: CoreUsage[];
}

export interface MemoryMetrics {
  total_bytes: number;
  used_bytes: number;
  free_bytes: number;
  usage_percent: number;
  swap_total_bytes: number;
  swap_used_bytes: number;
  swap_usage_percent: number;
}

export interface DiskMetrics {
  name: string;
  mount_point: string;
  total_space: number;
  available_space: number;
  used_space: number;
  usage_percent: number;
  file_system: string;
  is_removable: boolean;
}

export interface NetworkMetrics {
  upload_speed_bps: number;
  download_speed_bps: number;
  total_transmitted_bytes: number;
  total_received_bytes: number;
}

export interface TempMetric {
  label: string;
  temperature: number;
  critical?: number;
}

export interface SystemStaticInfo {
  host_name: string;
  os_name: string;
  os_version: string;
  kernel_version: string;
  uptime_seconds: number;
  cpu_arch: string;
}

export interface SystemOverview {
  timestamp: number;
  system_info: SystemStaticInfo;
  cpu: CpuMetrics;
  memory: MemoryMetrics;
  disks: DiskMetrics[];
  network: NetworkMetrics;
  temperatures: TempMetric[];
}

export interface ProcessInfo {
  pid: number;
  name: string;
  cpu_usage: number;
  memory_bytes: number;
  memory_mb: number;
  status: string;
  run_time_secs: number;
}

export interface PortInfo {
  port: number;
  protocol: string;
  local_address: string;
  state: string;
  pid?: number;
  process_name?: string;
}

export interface LatencyResult {
  target: string;
  success: boolean;
  rtt_ms: number;
  error?: string;
}
