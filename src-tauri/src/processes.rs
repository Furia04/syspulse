use sysinfo::{Pid, ProcessesToUpdate, ProcessStatus};
use crate::models::ProcessInfo;
use crate::telemetry::TelemetryState;

pub fn get_processes(state: &TelemetryState) -> Vec<ProcessInfo> {
    let mut sys = state.system.lock().unwrap();
    sys.refresh_processes(ProcessesToUpdate::All, true);

    let mut list: Vec<ProcessInfo> = sys
        .processes()
        .iter()
        .map(|(&pid, p)| {
            let mem_bytes = p.memory();
            let status_str = match p.status() {
                ProcessStatus::Run => "Running",
                ProcessStatus::Sleep => "Sleeping",
                ProcessStatus::Idle => "Idle",
                ProcessStatus::Stop => "Stopped",
                ProcessStatus::Zombie => "Zombie",
                _ => "Active",
            };

            ProcessInfo {
                pid: pid.as_u32(),
                name: p.name().to_string_lossy().to_string(),
                cpu_usage: p.cpu_usage(),
                memory_bytes: mem_bytes,
                memory_mb: (mem_bytes as f64) / (1024.0 * 1024.0),
                status: status_str.to_string(),
                run_time_secs: p.run_time(),
            }
        })
        .collect();

    // Sort default by CPU usage descending, then by memory
    list.sort_by(|a, b| {
        b.cpu_usage
            .partial_cmp(&a.cpu_usage)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| b.memory_bytes.cmp(&a.memory_bytes))
    });

    list
}

pub fn kill_process(state: &TelemetryState, pid: u32) -> Result<bool, String> {
    let mut sys = state.system.lock().unwrap();
    sys.refresh_processes(ProcessesToUpdate::All, true);

    let sys_pid = Pid::from_u32(pid);
    if let Some(process) = sys.process(sys_pid) {
        if process.kill() {
            Ok(true)
        } else {
            Err(format!(
                "No se pudo terminar el proceso PID {} (permisos insuficientes o proceso protegido)",
                pid
            ))
        }
    } else {
        Err(format!("Proceso con PID {} no encontrado", pid))
    }
}
