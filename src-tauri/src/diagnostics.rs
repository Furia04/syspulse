use std::net::{TcpStream, ToSocketAddrs};
use std::os::windows::process::CommandExt;
use std::process::Command;
use std::time::{Duration, Instant};
use sysinfo::Pid;

use crate::models::{LatencyResult, PortInfo};
use crate::telemetry::TelemetryState;

const CREATE_NO_WINDOW: u32 = 0x08000000;

pub fn ping_host(host_or_ip: &str) -> LatencyResult {
    let clean_host = host_or_ip.trim();
    if clean_host.is_empty() {
        return LatencyResult {
            target: host_or_ip.to_string(),
            success: false,
            rtt_ms: 0.0,
            error: Some("Destino no especificado".to_string()),
        };
    }

    // Try TCP connect first if port is specified, or default to 53/443/80
    let targets_to_try = if clean_host.contains(':') {
        vec![clean_host.to_string()]
    } else {
        vec![
            format!("{}:443", clean_host),
            format!("{}:80", clean_host),
            format!("{}:53", clean_host),
        ]
    };

    let start = Instant::now();

    for target in &targets_to_try {
        if let Ok(mut addrs) = target.to_socket_addrs() {
            if let Some(addr) = addrs.next() {
                let connect_start = Instant::now();
                if TcpStream::connect_timeout(&addr, Duration::from_millis(2000)).is_ok() {
                    let rtt = connect_start.elapsed().as_secs_f64() * 1000.0;
                    return LatencyResult {
                        target: clean_host.to_string(),
                        success: true,
                        rtt_ms: (rtt * 100.0).round() / 100.0,
                        error: None,
                    };
                }
            }
        }
    }

    // If TCP connect direct failed, fallback to native Windows ping (ICMP)
    let ping_output = Command::new("ping")
        .args(["-n", "1", "-w", "1500", clean_host])
        .creation_flags(CREATE_NO_WINDOW)
        .output();

    match ping_output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            if out.status.success() {
                // Parse "tiempo=XXms" or "time=XXms" or "tiempo<1ms"
                let rtt = parse_ping_rtt(&stdout).unwrap_or_else(|| {
                    start.elapsed().as_secs_f64() * 1000.0
                });
                LatencyResult {
                    target: clean_host.to_string(),
                    success: true,
                    rtt_ms: (rtt * 100.0).round() / 100.0,
                    error: None,
                }
            } else {
                LatencyResult {
                    target: clean_host.to_string(),
                    success: false,
                    rtt_ms: 0.0,
                    error: Some("Tiempo de espera agotado o host inaccesible".to_string()),
                }
            }
        }
        Err(e) => LatencyResult {
            target: clean_host.to_string(),
            success: false,
            rtt_ms: 0.0,
            error: Some(format!("Error al ejecutar diagnóstico: {}", e)),
        },
    }
}

fn parse_ping_rtt(output: &str) -> Option<f64> {
    for line in output.lines() {
        let lower = line.to_lowercase();
        if lower.contains("tiempo=") || lower.contains("time=") {
            let parts: Vec<&str> = if lower.contains("tiempo=") {
                lower.split("tiempo=").collect()
            } else {
                lower.split("time=").collect()
            };
            if let Some(right) = parts.get(1) {
                let num_str: String = right
                    .chars()
                    .take_while(|c| c.is_ascii_digit() || *c == '.')
                    .collect();
                if let Ok(val) = num_str.parse::<f64>() {
                    return Some(val);
                }
            }
        } else if lower.contains("tiempo<1ms") || lower.contains("time<1ms") {
            return Some(0.8);
        }
    }
    None
}

pub fn list_open_ports(state: &TelemetryState) -> Vec<PortInfo> {
    let mut ports = Vec::new();

    let output = Command::new("netstat")
        .args(["-ano", "-p", "tcp"])
        .creation_flags(CREATE_NO_WINDOW)
        .output();

    if let Ok(out) = output {
        let stdout = String::from_utf8_lossy(&out.stdout);
        let sys = state.system.lock().unwrap();

        for line in stdout.lines() {
            let tokens: Vec<&str> = line.split_whitespace().collect();
            // Typical line: TCP  0.0.0.0:135  0.0.0.0:0  LISTENING  944
            if tokens.len() >= 4 && tokens[0].eq_ignore_ascii_case("TCP") {
                let local_addr = tokens[1];
                let state_str = tokens[3];

                if state_str.eq_ignore_ascii_case("LISTENING") || state_str.eq_ignore_ascii_case("LISTON") {
                    let pid: Option<u32> = tokens.get(4).and_then(|p| p.parse().ok());
                    let port = local_addr
                        .rsplit(':')
                        .next()
                        .and_then(|p| p.parse::<u16>().ok())
                        .unwrap_or(0);

                    if port > 0 {
                        let proc_name = pid.and_then(|p| {
                            sys.process(Pid::from_u32(p))
                                .map(|proc| proc.name().to_string_lossy().to_string())
                        });

                        ports.push(PortInfo {
                            port,
                            protocol: "TCP".to_string(),
                            local_address: local_addr.to_string(),
                            state: "LISTENING".to_string(),
                            pid,
                            process_name: proc_name,
                        });
                    }
                }
            }
        }
    }

    // Sort by port number ascending
    ports.sort_by_key(|p| p.port);
    ports.dedup_by(|a, b| a.port == b.port && a.pid == b.pid);

    ports
}
