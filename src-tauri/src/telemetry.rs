use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use sysinfo::{Components, Disks, Networks, System};
use tauri::{AppHandle, Emitter};

use crate::models::{
    CoreUsage, CpuMetrics, DiskMetrics, MemoryMetrics, NetworkMetrics, SystemOverview,
    SystemStaticInfo, TempMetric,
};

pub struct TelemetryState {
    pub system: Mutex<System>,
    pub disks: Mutex<Disks>,
    pub networks: Mutex<Networks>,
    pub components: Mutex<Components>,
    last_network_tick: Mutex<Instant>,
}

impl TelemetryState {
    pub fn new() -> Self {
        let mut sys = System::new();
        sys.refresh_cpu_all();
        sys.refresh_memory();
        let disks = Disks::new_with_refreshed_list();
        let mut networks = Networks::new_with_refreshed_list();
        networks.refresh(true);
        let components = Components::new();

        Self {
            system: Mutex::new(sys),
            disks: Mutex::new(disks),
            networks: Mutex::new(networks),
            components: Mutex::new(components),
            last_network_tick: Mutex::new(Instant::now()),
        }
    }

    pub fn snapshot(&self) -> SystemOverview {
        let mut sys = self.system.lock().unwrap();
        let mut disks = self.disks.lock().unwrap();
        let mut networks = self.networks.lock().unwrap();
        let mut components = self.components.lock().unwrap();
        let mut last_tick = self.last_network_tick.lock().unwrap();

        // Refresh dynamic metrics
        sys.refresh_cpu_all();
        sys.refresh_memory();
        disks.refresh(true);
        networks.refresh(true);
        components.refresh(true);

        let elapsed = last_tick.elapsed().as_secs_f64().max(0.1);
        *last_tick = Instant::now();

        // CPU
        let cpus = sys.cpus();
        let global_usage = sys.global_cpu_usage();
        let brand = cpus
            .first()
            .map(|c| c.brand().to_string())
            .unwrap_or_else(|| "Unknown CPU".to_string());

        let cores: Vec<CoreUsage> = cpus
            .iter()
            .enumerate()
            .map(|(id, c)| CoreUsage {
                id,
                usage: c.cpu_usage(),
                frequency: c.frequency(),
            })
            .collect();

        let cpu_metrics = CpuMetrics {
            global_usage,
            core_count: cores.len(),
            brand,
            cores,
        };

        // Memory
        let total_bytes = sys.total_memory();
        let used_bytes = sys.used_memory();
        let free_bytes = sys.free_memory();
        let usage_percent = if total_bytes > 0 {
            (used_bytes as f32 / total_bytes as f32) * 100.0
        } else {
            0.0
        };

        let swap_total_bytes = sys.total_swap();
        let swap_used_bytes = sys.used_swap();
        let swap_usage_percent = if swap_total_bytes > 0 {
            (swap_used_bytes as f32 / swap_total_bytes as f32) * 100.0
        } else {
            0.0
        };

        let memory_metrics = MemoryMetrics {
            total_bytes,
            used_bytes,
            free_bytes,
            usage_percent,
            swap_total_bytes,
            swap_used_bytes,
            swap_usage_percent,
        };

        // Disks
        let disk_list: Vec<DiskMetrics> = disks
            .iter()
            .map(|d| {
                let total = d.total_space();
                let available = d.available_space();
                let used = total.saturating_sub(available);
                let percent = if total > 0 {
                    (used as f32 / total as f32) * 100.0
                } else {
                    0.0
                };

                DiskMetrics {
                    name: d.name().to_string_lossy().to_string(),
                    mount_point: d.mount_point().to_string_lossy().to_string(),
                    total_space: total,
                    available_space: available,
                    used_space: used,
                    usage_percent: percent,
                    file_system: d.file_system().to_string_lossy().to_string(),
                    is_removable: d.is_removable(),
                }
            })
            .collect();

        // Network calculation
        let mut total_rx = 0;
        let mut total_tx = 0;
        for (_, net_data) in networks.iter() {
            total_rx += net_data.received();
            total_tx += net_data.transmitted();
        }

        let download_bps = (total_rx as f64 / elapsed) as u64;
        let upload_bps = (total_tx as f64 / elapsed) as u64;

        let mut cumulative_rx = 0;
        let mut cumulative_tx = 0;
        for (_, net_data) in networks.iter() {
            cumulative_rx += net_data.total_received();
            cumulative_tx += net_data.total_transmitted();
        }

        let network_metrics = NetworkMetrics {
            download_speed_bps: download_bps,
            upload_speed_bps: upload_bps,
            total_received_bytes: cumulative_rx,
            total_transmitted_bytes: cumulative_tx,
        };

        // Temperatures
        let temperatures: Vec<TempMetric> = components
            .iter()
            .map(|c| TempMetric {
                label: c.label().to_string(),
                temperature: c.temperature().unwrap_or(0.0),
                critical: c.critical(),
            })
            .collect();

        // System static metadata
        let system_info = SystemStaticInfo {
            host_name: System::host_name().unwrap_or_else(|| "Unknown-Host".to_string()),
            os_name: System::name().unwrap_or_else(|| "Windows".to_string()),
            os_version: System::os_version().unwrap_or_else(|| "".to_string()),
            kernel_version: System::kernel_version().unwrap_or_else(|| "".to_string()),
            uptime_seconds: System::uptime(),
            cpu_arch: System::cpu_arch(),
        };

        SystemOverview {
            timestamp: chrono::Utc::now().timestamp_millis(),
            system_info,
            cpu: cpu_metrics,
            memory: memory_metrics,
            disks: disk_list,
            network: network_metrics,
            temperatures,
        }
    }
}

pub fn spawn_telemetry_loop(app_handle: AppHandle, state: Arc<TelemetryState>) {
    std::thread::spawn(move || {
        // Safe to query WMI/components in worker thread
        {
            let mut comps = state.components.lock().unwrap();
            *comps = Components::new_with_refreshed_list();
        }

        loop {
            std::thread::sleep(Duration::from_millis(1000));
            let metrics = state.snapshot();
            if let Err(e) = app_handle.emit("metrics-stream", &metrics) {
                log::warn!("Failed to emit telemetry stream: {:?}", e);
            }
        }
    });
}
