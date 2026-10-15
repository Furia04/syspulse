mod diagnostics;
mod exporter;
mod models;
mod processes;
mod telemetry;

use std::sync::Arc;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager, State,
};

use crate::models::{LatencyResult, PortInfo, ProcessInfo, SystemOverview, SystemReport};
use crate::telemetry::{spawn_telemetry_loop, TelemetryState};

#[tauri::command]
fn get_initial_metrics(state: State<'_, Arc<TelemetryState>>) -> SystemOverview {
    state.snapshot()
}

#[tauri::command]
fn get_process_list(state: State<'_, Arc<TelemetryState>>) -> Vec<ProcessInfo> {
    processes::get_processes(&state)
}

#[tauri::command]
fn kill_process(state: State<'_, Arc<TelemetryState>>, pid: u32) -> Result<bool, String> {
    processes::kill_process(&state, pid)
}

#[tauri::command]
fn get_open_ports(state: State<'_, Arc<TelemetryState>>) -> Vec<PortInfo> {
    diagnostics::list_open_ports(&state)
}

#[tauri::command]
fn run_ping_test(host: String) -> LatencyResult {
    diagnostics::ping_host(&host)
}

#[tauri::command]
fn export_system_report(
    state: State<'_, Arc<TelemetryState>>,
    format: String,
) -> Result<String, String> {
    let overview = state.snapshot();
    let top_processes = processes::get_processes(&state);
    let open_ports = diagnostics::list_open_ports(&state);

    let report = SystemReport {
        generated_at: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        overview,
        top_processes,
        open_ports,
    };

    let filename = format!(
        "syspulse_report_{}.{}",
        chrono::Local::now().format("%Y%m%d_%H%M%S"),
        if format == "json" { "json" } else { "html" }
    );

    let target_dir = std::env::var("USERPROFILE")
        .map(|p| std::path::PathBuf::from(p).join("Downloads"))
        .unwrap_or_else(|_| std::env::current_dir().unwrap_or_default());

    let target_path = target_dir.join(&filename);

    let content = if format == "json" {
        serde_json::to_string_pretty(&report).map_err(|e| e.to_string())?
    } else {
        exporter::generate_html_report(&report)
    };

    std::fs::write(&target_path, &content).map_err(|e| format!("Error guardando archivo: {}", e))?;

    Ok(target_path.to_string_lossy().to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let telemetry_state = Arc::new(TelemetryState::new());

    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .manage(Arc::clone(&telemetry_state))
        .setup({
            let telemetry_state = Arc::clone(&telemetry_state);
            move |app| {
                // Tray Menu configuration
                let show_i = MenuItem::with_id(app, "show", "Mostrar SysPulse", true, None::<&str>)?;
                let quit_i = MenuItem::with_id(app, "quit", "Salir de SysPulse", true, None::<&str>)?;
                let menu = Menu::with_items(app, &[&show_i, &quit_i])?;

                let _tray = TrayIconBuilder::new()
                    .icon(app.default_window_icon().unwrap().clone())
                    .menu(&menu)
                    .show_menu_on_left_click(false)
                    .on_menu_event(|app, event| match event.id.as_ref() {
                        "quit" => {
                            app.exit(0);
                        }
                        "show" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                        _ => {}
                    })
                    .on_tray_icon_event(|tray, event| {
                        if let TrayIconEvent::Click {
                            button: MouseButton::Left,
                            button_state: MouseButtonState::Up,
                            ..
                        } = event
                        {
                            let app = tray.app_handle();
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                    })
                    .build(app)?;

                // Start background telemetry streaming
                spawn_telemetry_loop(app.handle().clone(), telemetry_state);

                Ok(())
            }
        })
        .invoke_handler(tauri::generate_handler![
            get_initial_metrics,
            get_process_list,
            kill_process,
            get_open_ports,
            run_ping_test,
            export_system_report
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_telemetry_snapshot() {
        let state = TelemetryState::new();
        let snapshot = state.snapshot();
        assert!(snapshot.cpu.core_count > 0, "CPU core count should be > 0");
        assert!(snapshot.memory.total_bytes > 0, "Total RAM should be > 0");
        assert!(!snapshot.system_info.os_name.is_empty(), "OS name should not be empty");
    }

    #[test]
    fn test_diagnostics_ping_localhost() {
        let result = diagnostics::ping_host("127.0.0.1");
        assert_eq!(result.target, "127.0.0.1");
        assert!(result.success, "Ping to 127.0.0.1 should succeed");
    }

    #[test]
    fn test_html_report_generation() {
        let state = TelemetryState::new();
        let overview = state.snapshot();
        let report = SystemReport {
            generated_at: "2026-10-15 00:00:00".to_string(),
            overview,
            top_processes: Vec::new(),
            open_ports: Vec::new(),
        };
        let html = exporter::generate_html_report(&report);
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("SysPulse - Informe de Diagnóstico"));
    }
}
