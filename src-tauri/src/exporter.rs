use crate::models::SystemReport;

pub fn generate_html_report(report: &SystemReport) -> String {
    let ov = &report.overview;
    let ram_used_gb = ov.memory.used_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
    let ram_total_gb = ov.memory.total_bytes as f64 / (1024.0 * 1024.0 * 1024.0);

    let mut disks_html = String::new();
    for d in &ov.disks {
        let total_gb = d.total_space as f64 / (1024.0 * 1024.0 * 1024.0);
        let used_gb = d.used_space as f64 / (1024.0 * 1024.0 * 1024.0);
        disks_html.push_str(&format!(
            r#"<div class="card">
                <div class="card-header">
                    <span><strong>{}</strong> ({})</span>
                    <span>{:.1}%</span>
                </div>
                <div class="bar-container">
                    <div class="bar-fill" style="width: {:.1}%;"></div>
                </div>
                <div class="subtext">{:.1} GB usados de {:.1} GB ({})</div>
            </div>"#,
            d.name, d.mount_point, d.usage_percent, d.usage_percent, used_gb, total_gb, d.file_system
        ));
    }

    let mut proc_rows = String::new();
    for p in report.top_processes.iter().take(25) {
        proc_rows.push_str(&format!(
            r#"<tr>
                <td>{}</td>
                <td><strong>{}</strong></td>
                <td>{:.1}%</td>
                <td>{:.1} MB</td>
                <td><span class="badge">{}</span></td>
            </tr>"#,
            p.pid, p.name, p.cpu_usage, p.memory_mb, p.status
        ));
    }

    let mut port_rows = String::new();
    for port in report.open_ports.iter().take(25) {
        let proc = port.process_name.as_deref().unwrap_or("Sistema / N/A");
        let pid_str = port.pid.map(|p| p.to_string()).unwrap_or_else(|| "-".to_string());
        port_rows.push_str(&format!(
            r#"<tr>
                <td><strong>:{}</strong></td>
                <td>{}</td>
                <td>{}</td>
                <td>{}</td>
                <td><span class="badge badge-green">{}</span></td>
            </tr>"#,
            port.port, port.protocol, proc, pid_str, port.state
        ));
    }

    format!(
        r#"<!DOCTYPE html>
<html lang="es">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Informe SysPulse - {}</title>
    <style>
        :root {{
            --bg: #0b0f19;
            --surface: #131a2a;
            --border: #1e293b;
            --text: #e2e8f0;
            --muted: #94a3b8;
            --accent: #38bdf8;
            --accent-green: #34d399;
            --accent-orange: #fb923c;
        }}
        * {{ box-sizing: border-box; margin: 0; padding: 0; }}
        body {{
            background: var(--bg);
            color: var(--text);
            font-family: ui-sans-serif, system-ui, -apple-system, sans-serif;
            padding: 2rem;
            line-height: 1.5;
        }}
        .header {{
            border-bottom: 1px solid var(--border);
            padding-bottom: 1.5rem;
            margin-bottom: 2rem;
            display: flex;
            justify-content: space-between;
            align-items: center;
        }}
        h1 {{ font-size: 1.75rem; color: var(--accent); display: flex; align-items: center; gap: 0.5rem; }}
        .badge {{
            background: #1e293b;
            color: var(--muted);
            padding: 0.2rem 0.6rem;
            border-radius: 9999px;
            font-size: 0.75rem;
            font-weight: 600;
        }}
        .badge-green {{ background: rgba(52, 211, 153, 0.15); color: var(--accent-green); }}
        .grid {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
            gap: 1rem;
            margin-bottom: 2rem;
        }}
        .card {{
            background: var(--surface);
            border: 1px solid var(--border);
            border-radius: 0.75rem;
            padding: 1.25rem;
        }}
        .card-title {{ font-size: 0.85rem; color: var(--muted); text-transform: uppercase; font-weight: 600; margin-bottom: 0.5rem; }}
        .metric {{ font-size: 1.75rem; font-weight: 700; color: var(--text); }}
        .subtext {{ font-size: 0.8rem; color: var(--muted); margin-top: 0.4rem; }}
        .bar-container {{
            background: #0f172a;
            border-radius: 9999px;
            height: 8px;
            overflow: hidden;
            margin: 0.5rem 0;
        }}
        .bar-fill {{
            background: var(--accent);
            height: 100%;
            border-radius: 9999px;
        }}
        .section-title {{
            font-size: 1.2rem;
            font-weight: 600;
            margin: 2rem 0 1rem;
            color: var(--text);
            display: flex;
            align-items: center;
            gap: 0.5rem;
        }}
        table {{
            width: 100%;
            border-collapse: collapse;
            background: var(--surface);
            border: 1px solid var(--border);
            border-radius: 0.75rem;
            overflow: hidden;
            margin-bottom: 2rem;
            font-size: 0.9rem;
        }}
        th, td {{ padding: 0.75rem 1rem; text-align: left; border-bottom: 1px solid var(--border); }}
        th {{ background: #0f172a; color: var(--muted); font-weight: 600; text-transform: uppercase; font-size: 0.75rem; }}
        tr:last-child td {{ border-bottom: none; }}
        tr:hover td {{ background: rgba(255, 255, 255, 0.02); }}
    </style>
</head>
<body>
    <div class="header">
        <div>
            <h1>⚡ SysPulse - Informe de Diagnóstico</h1>
            <p class="subtext">Host: <strong>{}</strong> | SO: {} {} ({}) | Kernel: {}</p>
        </div>
        <div style="text-align: right;">
            <span class="badge">Generado: {}</span>
            <p class="subtext">Tiempo activo: {} horas</p>
        </div>
    </div>

    <div class="grid">
        <div class="card">
            <div class="card-title">Carga Global de CPU</div>
            <div class="metric">{:.1}%</div>
            <div class="bar-container">
                <div class="bar-fill" style="width: {:.1}%;"></div>
            </div>
            <div class="subtext">{} ({} núcleos)</div>
        </div>
        <div class="card">
            <div class="card-title">Memoria RAM</div>
            <div class="metric">{:.1}%</div>
            <div class="bar-container">
                <div class="bar-fill" style="width: {:.1}%;"></div>
            </div>
            <div class="subtext">{:.1} GB usados de {:.1} GB</div>
        </div>
        <div class="card">
            <div class="card-title">Throughput de Red</div>
            <div class="metric">↓ {:.1} KB/s</div>
            <div class="subtext">↑ {:.1} KB/s transmisión actual</div>
        </div>
        <div class="card">
            <div class="card-title">Memoria Swap</div>
            <div class="metric">{:.1}%</div>
            <div class="subtext">{:.1} GB usados</div>
        </div>
    </div>

    <div class="section-title">💾 Almacenamiento y Unidades</div>
    <div class="grid">
        {}
    </div>

    <div class="section-title">📊 Procesos con Mayor Carga de Trabajo (Top 25)</div>
    <table>
        <thead>
            <tr>
                <th>PID</th>
                <th>Nombre</th>
                <th>% CPU</th>
                <th>Memoria</th>
                <th>Estado</th>
            </tr>
        </thead>
        <tbody>
            {}
        </tbody>
    </table>

    <div class="section-title">🔌 Puertos Locales en Escucha (Sockets TCP)</div>
    <table>
        <thead>
            <tr>
                <th>Puerto</th>
                <th>Protocolo</th>
                <th>Proceso</th>
                <th>PID</th>
                <th>Estado</th>
            </tr>
        </thead>
        <tbody>
            {}
        </tbody>
    </table>
</body>
</html>"#,
        report.generated_at,
        ov.system_info.host_name,
        ov.system_info.os_name,
        ov.system_info.os_version,
        ov.system_info.cpu_arch,
        ov.system_info.kernel_version,
        report.generated_at,
        ov.system_info.uptime_seconds / 3600,
        ov.cpu.global_usage,
        ov.cpu.global_usage,
        ov.cpu.brand,
        ov.cpu.core_count,
        ov.memory.usage_percent,
        ov.memory.usage_percent,
        ram_used_gb,
        ram_total_gb,
        ov.network.download_speed_bps as f64 / 1024.0,
        ov.network.upload_speed_bps as f64 / 1024.0,
        ov.memory.swap_usage_percent,
        ov.memory.swap_used_bytes as f64 / (1024.0 * 1024.0 * 1024.0),
        disks_html,
        proc_rows,
        port_rows
    )
}
