use eframe::egui;
use eframe::App;
use egui_plot::{Plot, Line, PlotPoints};
use sysinfo::System;

use std::time::{Duration, Instant};

pub struct MonitorApp {
    sys: System,
    last_update: Instant,
    cpu_history: Vec<f32>,
    ram_history: Vec<f32>,
}

impl Default for MonitorApp {
    fn default() -> Self {
        let mut sys = System::new_all(); // Carga todos los datos disponibles
        sys.refresh_all();

        Self {
            sys,
            last_update: Instant::now(),
            cpu_history: vec![0.0; 60],
            ram_history: vec![0.0; 60],
        }
    }
}

impl App for MonitorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.last_update.elapsed() > Duration::from_secs(1) {
            self.sys.refresh_all();

            let cpus = self.sys.cpus();
            let cpu_usage = cpus.iter().map(|c| c.cpu_usage()).sum::<f32>() / cpus.len() as f32;

            let total_memory = self.sys.total_memory();
            let used_memory = self.sys.used_memory();
            let ram_usage = if total_memory > 0 {
                (used_memory as f32 / total_memory as f32) * 100.0
            } else {
                0.0
            };

            self.cpu_history.push(cpu_usage);
            self.cpu_history.remove(0);
            self.ram_history.push(ram_usage);
            self.ram_history.remove(0);

            self.last_update = Instant::now();
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Monitor de Sistema");

            ui.label(format!("CPU: {:.2}%", self.cpu_history.last().unwrap()));
            Plot::new("cpu_plot").view_aspect(2.0).show(ui, |plot_ui| {
                plot_ui.line(Line::new(PlotPoints::from_iter(
                    self.cpu_history
                        .iter()
                        .enumerate()
                        .map(|(i, v)| [i as f64, *v as f64]),
                ))
                .color(egui::Color32::LIGHT_BLUE)
                .name("CPU %"));
            });

            ui.label(format!("RAM: {:.2}%", self.ram_history.last().unwrap()));
            Plot::new("ram_plot").view_aspect(2.0).show(ui, |plot_ui| {
                plot_ui.line(Line::new(PlotPoints::from_iter(
                    self.ram_history
                        .iter()
                        .enumerate()
                        .map(|(i, v)| [i as f64, *v as f64]),
                ))
                .color(egui::Color32::LIGHT_GREEN)
                .name("RAM %"));
            });

            // 👇 Esto queda desactivado porque sysinfo ya no da datos de red
            // ui.label("Red: No disponible en sysinfo 0.35.0. Usa netstat2 o pnet si lo necesitas.");
        });

        ctx.request_repaint_after(Duration::from_millis(100));
    }
}
