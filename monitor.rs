use sysinfo::{System, Networks, Disks, CpuExt, NetworkExt, DiskExt};
use std::{thread::sleep, time::Duration};

pub struct Monitor {
    sys: System,
}

impl Monitor {
    pub fn new() -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();
        Self { sys }
    }

    pub fn update_metrics(&mut self) {
        self.sys.refresh_all();
    }

    pub fn get_cpu_usage(&self) -> f32 {
        let total_usage: f32 = self.sys.cpus().iter().map(|cpu| cpu.cpu_usage()).sum();
        total_usage / self.sys.cpus().len() as f32
    }

    pub fn get_memory_usage(&self) -> (u64, u64) {
        (self.sys.total_memory(), self.sys.used_memory())
    }

    pub fn get_network_usage(&self) -> (f64, f64) {
        let networks = Networks::new_with_refreshed_list();
        let wifi = networks.get("Wi-Fi");
        if let Some(net) = wifi {
            let rx_mbps = (net.total_received() as f64 * 8.0) / (1024.0 * 1024.0);
            let tx_mbps = (net.total_transmitted() as f64 * 8.0) / (1024.0 * 1024.0);
            return (rx_mbps, tx_mbps);
        }
        (0.0, 0.0)
    }

    pub fn get_disk_usage(&self) -> Vec<String> {
        let disks = Disks::new_with_refreshed_list();
        disks.iter().map(|disk| {
            format!(
                "[{}] Tipo: {:?} | Lectura: {:.2} MB | Escritura: {:.2} MB",
                disk.mount_point().to_string_lossy(),
                disk.kind(),
                disk.usage().read_bytes as f64 / (1024.0 * 1024.0),
                disk.usage().written_bytes as f64 / (1024.0 * 1024.0)
            )
        }).collect()
    }
}