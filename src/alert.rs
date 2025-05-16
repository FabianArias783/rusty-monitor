use notify_rust::Notification;
use std::time::{Instant, Duration};

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

    pub fn check_alerts(&mut self, cpu: f32, net: f32, mem: f32) {
        let alerts = self.alerts.clone(); // evitar préstamos conflictivos

        for alert in alerts {
            if cpu > alert.cpu_threshold || net > alert.network_threshold || mem > alert.memory_threshold {
                self.trigger_alert(&alert, cpu, net, mem);
            }
        }
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
             Red: {:.2} MB (Umbral: {:.2} MB)",
            cpu, alert.cpu_threshold, mem, alert.memory_threshold, net, alert.network_threshold
        );

        let _ = Notification::new()
            .summary("⚠️ ¡Alerta del sistema!")
            .body(&format!(
                "CPU: {:.2}% (>{:.2}%)\nMemoria: {:.2}% (>{:.2}%)\nRed: {:.2} MB (>{:.2} MB)",
                cpu, alert.cpu_threshold,
                mem, alert.memory_threshold,
                net, alert.network_threshold
            ))
            .show();
    }
}