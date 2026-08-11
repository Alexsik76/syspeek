use sysinfo::System;

pub struct SystemMetrics {
    pub cpu_usage: u8,
    pub memory_usage: String,
}

pub struct MetricsCollector {
    system: System,
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricsCollector {
    pub fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_all();
        Self { system }
    }

    pub fn fetch(&mut self) -> SystemMetrics {
        self.system.refresh_cpu_usage();
        self.system.refresh_memory();

        let cpu = self.system.global_cpu_usage() as u8;
        let mem_used = self.system.used_memory() as f64 / 1_073_741_824.0;
        let mem_total = self.system.total_memory() as f64 / 1_073_741_824.0;

        SystemMetrics {
            cpu_usage: cpu,
            memory_usage: format!("{:.1} GB / {:.1} GB", mem_used, mem_total),
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_fetch() {
        let mut collector = MetricsCollector::new();
        let metrics = collector.fetch();

        assert!(metrics.cpu_usage <= 100);
        assert!(!metrics.memory_usage.is_empty());
    }
}
