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
    top_cpu: Vec<(i32, String, f32)>, // (pid, nombre, cpu%)
    top_ram: Vec<(i32, String, f64)>, // (pid, nombre, memoria MB)
    dark_mode: bool,
}

impl Default for MonitorApp {
    fn default() -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();

        Self {
            sys,
            last_update: Instant::now(),
            cpu_history: vec![0.0; 60],
            ram_history: vec![0.0; 60],
            top_cpu: Vec::new(),
            top_ram: Vec::new(),
            dark_mode: true,
        }
    }
}

impl App for MonitorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.dark_mode {
            ctx.set_visuals(egui::Visuals::dark());
        } else {
            ctx.set_visuals(egui::Visuals::light());
        }

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

            let num_cores = cpus.len() as f32;

            let mut processes: Vec<_> = self.sys.processes().values().collect();

            // Top 5 CPU procesos normalizados por núcleos
            processes.sort_by(|a, b| b.cpu_usage().partial_cmp(&a.cpu_usage()).unwrap());
            self.top_cpu = processes.iter()
                .take(5)
                .map(|p| {
                    let cpu_pct = p.cpu_usage() / num_cores;
                    (p.pid().as_u32() as i32, p.name().to_string_lossy().to_string(), cpu_pct)
                })
                .collect();

            // Top 5 RAM en MB, y limitar valores razonables
            processes.sort_by(|a, b| b.memory().cmp(&a.memory()));
            self.top_ram = processes.iter()
                .take(5)
                .map(|p| {
                    let mem_mb = p.memory() as f64 / 1024.0 / 1024.0;
                    (p.pid().as_u32() as i32, p.name().to_string_lossy().to_string(), mem_mb)
                })
                .collect();


            self.last_update = Instant::now();
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Monitor de Sistema");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                    if ui.button(if self.dark_mode { "Modo Claro" } else { "Modo Oscuro" }).clicked() {
                        self.dark_mode = !self.dark_mode;
                    }
                });
            });

            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(format!("CPU: {:.2}%", self.cpu_history.last().unwrap()));
                    Plot::new("cpu_plot")
                        .view_aspect(2.0)
                        .height(250.0)
                        .show(ui, |plot_ui| {
                            plot_ui.line(Line::new(PlotPoints::from_iter(
                                self.cpu_history.iter().enumerate().map(|(i, v)| [i as f64, *v as f64]),
                            ))
                            .color(egui::Color32::LIGHT_BLUE)
                            .name("CPU %"));
                        });
                });

                ui.vertical(|ui| {
                    ui.label(format!("RAM: {:.2}%", self.ram_history.last().unwrap()));
                    Plot::new("ram_plot")
                        .view_aspect(2.0)
                        .height(250.0)
                        .show(ui, |plot_ui| {
                            plot_ui.line(Line::new(PlotPoints::from_iter(
                                self.ram_history.iter().enumerate().map(|(i, v)| [i as f64, *v as f64]),
                            ))
                            .color(egui::Color32::LIGHT_GREEN)
                            .name("RAM %"));
                        });
                });
            });

            ui.separator();

            ui.label("Top 5 Procesos por uso de CPU:");
            ui.push_id("cpu_scroll", |ui| {
                egui::ScrollArea::vertical().max_height(150.0).show(ui, |ui| {
                    egui::Grid::new("cpu_grid").striped(true).show(ui, |ui| {
                        ui.label("PID");
                        ui.label("Nombre");
                        ui.label("CPU %");
                        ui.end_row();
                        for (pid, name, cpu) in &self.top_cpu {
                            ui.label(pid.to_string());
                            ui.label(name);
                            ui.label(format!("{:.2}%", cpu));
                            ui.end_row();
                        }
                    });
                });
            });

            ui.separator();

            ui.label("Top 5 Procesos por uso de RAM:");
            ui.push_id("ram_scroll", |ui| {
                egui::ScrollArea::vertical().max_height(150.0).show(ui, |ui| {
                    egui::Grid::new("ram_grid").striped(true).show(ui, |ui| {
                        ui.label("PID");
                        ui.label("Nombre");
                        ui.label("RAM (MB)");
                        ui.end_row();
                        for (pid, name, ram) in &self.top_ram {
                            ui.label(pid.to_string());
                            ui.label(name);
                            ui.label(format!("{:.2} MB", ram));
                            ui.end_row();
                        }
                    });
                });
            });
        });

        ctx.request_repaint_after(Duration::from_millis(100));
    }
}
