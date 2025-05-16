use notify_rust::Notification;
use std::time::{Instant, Duration};
use sysinfo::System; // Importa solo System
use crate::osstr_to_string; // Necesitamos esta función desde main.rs

#[derive(Clone)]
pub struct AlertCondition {
    pub cpu_threshold: f32,
    pub memory_threshold: f32,
    pub network_threshold: f32,
}

pub struct AlertManager {
    alerts: Vec<AlertCondition>,
    last_alert: Option<Instant>,
}

impl AlertManager {
    pub fn new() -> Self {
        let mut manager = AlertManager {
            alerts: Vec::new(),
            last_alert: None,
        };

        // Umbrales por defecto
        manager.add_alert(AlertCondition {
            cpu_threshold: 80.0,
            memory_threshold: 80.0,
            network_threshold: 10.0,
        });

        manager
    }

    pub fn add_alert(&mut self, alert: AlertCondition) {
        self.alerts.push(alert);
    }

    pub fn check_alerts(&mut self, cpu: f32, net: f32, mem: f32, system: &System, num_cores: f32) {
        let alerts = self.alerts.clone(); // evitar préstamos conflictivos

        for alert in alerts {
            if cpu > alert.cpu_threshold {
                // Obtener el proceso con mayor uso de CPU
                let mut processes = system.processes().values().collect::<Vec<_>>();
                processes.sort_by(|a, b| b.cpu_usage().partial_cmp(&a.cpu_usage()).unwrap());
                if let Some(top_process) = processes.first() {
                    let top_cpu_percent = top_process.cpu_usage() / num_cores;
                    self.trigger_cpu_alert(&alert, cpu, &top_process.name().to_string_lossy(), top_cpu_percent);
                } else {
                    self.trigger_alert(&alert, cpu, net, mem); // Si no hay procesos, alerta genérica
                }
            } else if mem > alert.memory_threshold {
                // Obtener el proceso con mayor uso de memoria
                let mut processes_mem = system.processes().values().collect::<Vec<_>>();
                processes_mem.sort_by(|a, b| b.memory().cmp(&b.memory())); // Orden ascendente, invertimos
                if let Some(top_process_mem) = processes_mem.last() {
                    let mem_mb = top_process_mem.memory() as f64 / (1024.0 * 1024.0);
                    self.trigger_memory_alert(&alert, mem, &top_process_mem.name().to_string_lossy(), mem_mb);
                } else {
                    self.trigger_memory_alert(&alert, mem, "", 0.0); // Si no hay procesos
                }
            } else if net > alert.network_threshold {
                self.trigger_network_alert(&alert, net);
            }
        }
    }

    fn trigger_cpu_alert(&mut self, alert: &AlertCondition, cpu: f32, process_name: &str, process_cpu: f32) {
        let now = Instant::now();

        if let Some(last) = self.last_alert {
            if now.duration_since(last) < Duration::from_secs(60) {
                return; // Cooldown de 60 segundos
            }
        }

        self.last_alert = Some(now);

        println!(
            "⚠️ ALERTA: ¡Alto uso de CPU detectado!\n\
             Proceso: {} ({:.2}%)\n\
             Uso total de CPU: {:.2}% (Umbral: {:.2}%)",
            process_name, process_cpu, cpu, alert.cpu_threshold
        );

        let _ = Notification::new()
            .summary("⚠️ ¡Alerta de CPU!")
            .body(&format!(
                "Proceso: {} ({:.2}%)\nUso total de CPU: {:.2}% (>{:.2}%)",
                process_name, process_cpu, cpu, alert.cpu_threshold
            ))
            .show();
    }

    fn trigger_memory_alert(&mut self, alert: &AlertCondition, mem_percent: f32, process_name: &str, mem_mb: f64) {
        let now = Instant::now();

        if let Some(last) = self.last_alert {
            if now.duration_since(last) < Duration::from_secs(60) {
                return; // Cooldown de 60 segundos
            }
        }

        self.last_alert = Some(now);

        println!(
            "⚠️ ALERTA: ¡Alto uso de memoria detectado!\n\
             Proceso: {} ({:.2} MB)\n\
             Uso de memoria: {:.2}% (Umbral: {:.2}%)",
            process_name, mem_mb, mem_percent, alert.memory_threshold
        );

        let _ = Notification::new()
            .summary("⚠️ ¡Alerta de Memoria!")
            .body(&format!(
                "Proceso: {} ({:.2} MB)\nUso de memoria: {:.2}% (>{:.2}%)",
                process_name, mem_mb, mem_percent, alert.memory_threshold
            ))
            .show();
    }

    fn trigger_network_alert(&mut self, alert: &AlertCondition, net: f32) {
        let now = Instant::now();

        if let Some(last) = self.last_alert {
            if now.duration_since(last) < Duration::from_secs(60) {
                return; // Cooldown de 60 segundos
            }
        }

        self.last_alert = Some(now);

        println!(
            "⚠️ ALERTA: ¡Alto tráfico de red detectado!\n\
             Velocidad de red: {:.2} MB/s (Umbral: {:.2} MB/s)",
            net, alert.network_threshold
        );

        let _ = Notification::new()
            .summary("⚠️ ¡Alerta de Red!")
            .body(&format!(
                "Velocidad de red: {:.2} MB/s (>{:.2} MB/s)",
                net, alert.network_threshold
            ))
            .show();
    }

    fn trigger_alert(&mut self, alert: &AlertCondition, cpu: f32, net: f32, mem: f32) {
        let now = Instant::now();

        if let Some(last) = self.last_alert {
            if now.duration_since(last) < Duration::from_secs(60) {
                return; // Cooldown de 60 segundos
            }
        }

        self.last_alert = Some(now);

        println!(
            "⚠️ ALERTA: Condición crítica detectada!\n\
             CPU: {:.2}% (Umbral: {:.2}%)\n\
             Memoria: {:.2}% (Umbral: {:.2}%)\n\
             Red: {:.2} MB/s (Umbral: {:.2} MB/s)",
            cpu, alert.cpu_threshold, mem, alert.memory_threshold, net, alert.network_threshold
        );

        let _ = Notification::new()
            .summary("⚠️ ¡Alerta del sistema!")
            .body(&format!(
                "CPU: {:.2}% (>{:.2}%)\nMemoria: {:.2}% (>{:.2}%)\nRed: {:.2} MB/s (>{:.2} MB/s)",
                cpu, alert.cpu_threshold,
                mem, alert.memory_threshold,
                net, alert.network_threshold
            ))
            .show();
    }
}