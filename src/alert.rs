use notify_rust::Notification;
use std::time::{Instant, Duration};
use sysinfo::{System, Process};

#[derive(Clone, Debug)]
pub struct AlertCondition {
    pub cpu_threshold: f32,
    pub memory_threshold: f32,
    pub network_threshold: f32,
}

pub struct AlertManager {
    pub alerts: Vec<AlertCondition>,
    pub triggered_alerts: Vec<String>,
    last_cpu_alert: Option<Instant>,
    last_memory_alert: Option<Instant>,
    last_network_alert: Option<Instant>,
}

impl AlertManager {
    pub fn new() -> Self {
        AlertManager {
            alerts: vec![
                AlertCondition {
                    cpu_threshold: 80.0,
                    memory_threshold: 80.0,
                    network_threshold: 10.0,
                },
            ],
            triggered_alerts: Vec::new(),
            last_cpu_alert: None,
            last_memory_alert: None,
            last_network_alert: None,
        }
    }

    pub fn check_alerts(&mut self, cpu: f32, net: f32, mem: f32, system: &System, num_cores: f32) {
        let alerts = self.alerts.clone();

        for alert in alerts {
            if cpu > alert.cpu_threshold {
                let mut processes = system.processes().values().collect::<Vec<_>>();
                processes.sort_by(|a, b| b.cpu_usage().partial_cmp(&a.cpu_usage()).unwrap());
                if let Some(top_process) = processes.first() {
                    let top_cpu_percent = top_process.cpu_usage() / num_cores;
                    self.trigger_cpu_alert(&alert, cpu, &top_process.name().to_string_lossy(), top_cpu_percent);
                }
            }

            if mem > alert.memory_threshold {
                let mut processes_mem = system.processes().values().collect::<Vec<_>>();
                processes_mem.sort_by(|a, b| b.memory().cmp(&a.memory()));
                if let Some(top_process_mem) = processes_mem.first() {
                    let mem_mb = top_process_mem.memory() as f64 / (1024.0 * 1024.0);
                    self.trigger_memory_alert(&alert, mem, &top_process_mem.name().to_string_lossy(), mem_mb);
                }
            }

            if net > alert.network_threshold {
                self.trigger_network_alert(&alert, net);
            }
        }
    }

    fn trigger_cpu_alert(&mut self, alert: &AlertCondition, cpu: f32, process_name: &str, process_cpu: f32) {
        let now = Instant::now();

        if let Some(last) = self.last_cpu_alert {
            if now.duration_since(last) < Duration::from_secs(10) {
                return;
            }
        }

        self.last_cpu_alert = Some(now);
        let alert_message = format!(
            "⚠️ ALERTA: ¡Alto uso de CPU detectado!\nProceso: {} ({:.2}%)\nUso total de CPU: {:.2}% (Umbral: {:.2}%)",
            process_name, process_cpu, cpu, alert.cpu_threshold
        );
        println!("{}", alert_message);
        self.triggered_alerts.push(alert_message);

        let _ = Notification::new()
            .appname("Sistema de Defensa")
            .icon("warning")
            .summary("⚠️ ¡Alerta de CPU!")
            .body(&format!(
                "Proceso: {} ({:.2}%)\nUso total de CPU: {:.2}% (>{:.2}%)",
                process_name, process_cpu, cpu, alert.cpu_threshold
            ))
            .show();
    }

    fn trigger_memory_alert(&mut self, alert: &AlertCondition, mem_percent: f32, process_name: &str, mem_mb: f64) {
        let now = Instant::now();

        if let Some(last) = self.last_memory_alert {
            if now.duration_since(last) < Duration::from_secs(10) {
                return;
            }
        }

        self.last_memory_alert = Some(now);
        let alert_message = format!(
            "⚠️ ALERTA: ¡Alto uso de memoria detectado!\nProceso: {} ({:.2} MB)\nUso de memoria: {:.2}% (Umbral: {:.2}%)",
            process_name, mem_mb, mem_percent, alert.memory_threshold
        );
        println!("{}", alert_message);
        self.triggered_alerts.push(alert_message);

        let _ = Notification::new()
            .appname("Sistema de Defensa")
            .icon("warning")
            .summary("⚠️ ¡Alerta de Memoria!")
            .body(&format!(
                "Proceso: {} ({:.2} MB)\nUso de memoria: {:.2}% (>{:.2}%)",
                process_name, mem_mb, mem_percent, alert.memory_threshold
            ))
            .show();
    }

    fn trigger_network_alert(&mut self, alert: &AlertCondition, net: f32) {
        let now = Instant::now();

        if let Some(last) = self.last_network_alert {
            if now.duration_since(last) < Duration::from_secs(10) {
                return;
            }
        }

        self.last_network_alert = Some(now);
        let alert_message = format!(
            "⚠️ ALERTA: ¡Alto tráfico de red detectado!\nVelocidad de red: {:.2} MB/s (Umbral: {:.2} MB/s)",
            net, alert.network_threshold
        );
        println!("{}", alert_message);
        self.triggered_alerts.push(alert_message);

        let _ = Notification::new()
            .appname("Sistema de Defensa")
            .icon("warning")
            .summary("⚠️ ¡Alerta de Red!")
            .body(&format!(
                "Velocidad de red: {:.2} MB/s (>{:.2} MB/s)",
                net, alert.network_threshold
            ))
            .show();
    }
}