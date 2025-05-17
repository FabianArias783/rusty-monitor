use std::{fs::OpenOptions, io::{self, Write}, thread::{sleep, spawn}, time::Duration};
use std::env;
use chrono::Local;
use rusqlite::{Connection, Result, params};
use sysinfo::{System, Disks, Networks, ProcessesToUpdate, RefreshKind};
use std::ffi::OsStr;

use winreg::enums::*;
use winreg::RegKey;
use notify_rust::Notification;

mod alert;
use alert::AlertManager;

mod gui;

fn redirect_stdout() -> io::BufWriter<std::fs::File> {
    let log_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("metrics.log")
        .expect("Failed to open log file");

    io::BufWriter::new(log_file)
}

fn add_to_startup() {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let key = hkcu.open_subkey_with_flags(
        "Software\\Microsoft\\Windows\\CurrentVersion\\Run", 
        KEY_WRITE
    ).expect("Failed to open registry key");

    let exe_path_buf = env::current_exe()
        .expect("Failed to get current exe path");

    let exe_path = exe_path_buf
        .to_str()
        .expect("Failed to convert path to str");

    key.set_value("Metricas", &exe_path).expect("Failed to add program to startup");
}


fn osstr_to_string(os_str: &OsStr) -> String {
    os_str.to_string_lossy().into_owned()
}

fn run_monitoring() -> Result<()> {
    let mut log_writer = redirect_stdout();  // Redirigir la salida a un archivo de log
    add_to_startup();
    let mut alert_manager = AlertManager::new();

    let _ = Notification::new()
        .summary("Monitoreo iniciado")
        .body("Ya se está recopilando información del sistema.")
        .show();

    let conn = Connection::open("metrics.db")?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS metrics (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            Hora TEXT NOT NULL,
            Uso_CPU TEXT,
            Uso_CPU_Total TEXT,
            Memoria_total TEXT,
            Memoria_usada TEXT,
            Procesos TEXT,
            Discos TEXT,
            Internet TEXT,
            Procesos_CPU TEXT
        )",
        [],
    )?;

    let mut sys = System::new_with_specifics(RefreshKind::everything());
    let num_cores = sys.cpus().len() as f32;

    sys.refresh_cpu_all();
    sys.refresh_processes(ProcessesToUpdate::All, false);
    sleep(Duration::from_millis(500));

    loop {
        sys.refresh_cpu_all();
        sys.refresh_processes(ProcessesToUpdate::All, false);
        sys.refresh_memory();

        let mut networks = Networks::new_with_refreshed_list();
        let before_rx = networks.get("Wi-Fi").map(|d| d.total_received());
        let before_tx = networks.get("Wi-Fi").map(|d| d.total_transmitted());

        sleep(Duration::from_secs(5));

        networks.refresh(true);
        let after_rx = networks.get("Wi-Fi").map(|d| d.total_received());
        let after_tx = networks.get("Wi-Fi").map(|d| d.total_transmitted());

        let (received_mbps, transmitted_mbps) = match (before_rx, before_tx, after_rx, after_tx) {
            (Some(brx), Some(btx), Some(arx), Some(atx)) => {
                let down_bytes = arx.saturating_sub(brx);
                let up_bytes = atx.saturating_sub(btx);
                let down_mbps = (down_bytes as f64 * 8.0) / (1024.0 * 1024.0 * 5.0);
                let up_mbps = (up_bytes as f64 * 8.0) / (1024.0 * 1024.0 * 5.0);
                (down_mbps, up_mbps)
            }
            _ => (0.0, 0.0),
        };

        let total_memory = sys.total_memory();
        let used_memory = sys.used_memory();

        let total_cpu_usage: f32 = sys.cpus().iter().map(|c| c.cpu_usage()).sum();
        let avg_system_cpu_usage = total_cpu_usage / num_cores;

        let mut cpu_usages = Vec::new();
        for (i, cpu) in sys.cpus().iter().enumerate() {
            cpu_usages.push(format!("CPU {}: {:.2}%", i + 1, cpu.cpu_usage()));
        }
        let cpu_usage_str = cpu_usages.join(" | ");

        let mem_percentage = (used_memory as f64 / total_memory as f64) * 100.0;

        alert_manager.check_alerts(
            avg_system_cpu_usage,
            received_mbps as f32,
            mem_percentage as f32,
            &sys,
            num_cores,
        );

        let mut processes_cpu: Vec<_> = sys.processes().values().collect();
        processes_cpu.sort_by(|a, b| b.cpu_usage().partial_cmp(&a.cpu_usage()).unwrap());
        let mut process_cpu_info = Vec::new();
        for p in processes_cpu.iter().take(5) {
            let cpu_percent = p.cpu_usage() / num_cores;
            process_cpu_info.push(format!(
                "[{}] {} (CPU: {:.2}%)",
                p.pid(),
                osstr_to_string(p.name()),
                cpu_percent
            ));
        }
        let process_cpu_info_str = process_cpu_info.join(" | ");

        let mut procesos_mem: Vec<_> = sys.processes().values().collect();
        procesos_mem.sort_by(|a, b| b.memory().partial_cmp(&a.memory()).unwrap());
        let mut process_mem_info = Vec::new();
        for p in procesos_mem.iter().take(5) {
            process_mem_info.push(format!(
                "[{}] {} (Mem: {:.2} MB)",
                p.pid(),
                osstr_to_string(p.name()),
                p.memory() as f64 / (1024.0 * 1024.0)
            ));
        }
        let process_mem_info_str = process_mem_info.join(" | ");

        let disks = Disks::new_with_refreshed_list();
        let mut disk_info = Vec::new();
        for disk in disks.iter() {
            let kind = format!("{:?}", disk.kind());
            let mount = disk.mount_point().to_string_lossy();
            let read = disk.usage().read_bytes as f64 / (1024.0 * 1024.0);
            let written = disk.usage().written_bytes as f64 / (1024.0 * 1024.0);
            disk_info.push(format!(
                "[{}][Tipo: {}] Lectura: {:.2} MB | Escritura: {:.2} MB",
                mount, kind, read, written
            ));
        }
        let disk_info_str = disk_info.join(" | ");

        let network_info_str = format!(
            "Wi-Fi: {:.2} Mbps (down) / {:.2} Mbps (up)",
            received_mbps, transmitted_mbps
        );

        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

        let total_memory_mb = format!("{:.2} MB", total_memory as f64 / (1024.0 * 1024.0));
        let used_memory_mb = format!("{:.2} MB", used_memory as f64 / (1024.0 * 1024.0));

        conn.execute(
            "INSERT INTO metrics (
                Hora,
                Uso_CPU,
                Uso_CPU_Total,
                Memoria_total,
                Memoria_usada,
                Procesos,
                Discos,
                Internet,
                Procesos_CPU
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                timestamp,
                cpu_usage_str,
                format!("{:.2}", avg_system_cpu_usage),
                total_memory_mb,
                used_memory_mb,
                process_mem_info_str,
                disk_info_str,
                network_info_str,
                process_cpu_info_str
            ],
        )?;

        let output = format!(
            "=== Información del sistema ===\n\
             Fecha y hora: {}\n\
             CPU Uso (por núcleo): {}\n\
             CPU Uso (promedio total): {:.2}%\n\
             Memoria total: {}, Memoria usada: {}\n\
             Procesos más demandantes de CPU:\n{}\n\
             Procesos más demandantes de RAM:\n{}\n\
             Discos:\n{}\n\
             Redes:\n{}\n\
             Datos insertados en SQLite exitosamente.\n\
             =====================================\n\n",
            timestamp, cpu_usage_str, avg_system_cpu_usage, total_memory_mb, used_memory_mb, process_cpu_info_str, process_mem_info_str, disk_info_str, network_info_str
        );

        log_writer.write_all(output.as_bytes()).expect("Failed to write to log file");
        println!("{}", output);

        sleep(Duration::from_millis(500));
    }
}

fn main() -> Result<()> {
    // Lanzar el hilo de monitoreo (no bloqueante)
    spawn(|| {
        if let Err(e) = run_monitoring() {
            eprintln!("Error en monitoreo: {}", e);
        }
    });

    // Ejecutar la GUI en el hilo principal (sin spawn)
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default().with_inner_size([1040.0, 640.0]),
        ..Default::default()
    };
    let result = eframe::run_native(
        "Monitor de Sistema",
        options,
        Box::new(|_cc| Box::new(gui::MonitorApp::default())),
    );

    if let Err(err) = result {
        eprintln!("Error en la GUI: {}", err);
    }

    Ok(())
}
