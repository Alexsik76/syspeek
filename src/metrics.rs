use std::fmt;
use sysinfo::System;

/// Used vs. total RAM, in bytes. Displayed in GB via its `Display` impl so
/// callers can format it without allocating an intermediate `String`.
pub struct MemoryUsage {
    pub used_bytes: u64,
    pub total_bytes: u64,
}

impl fmt::Display for MemoryUsage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const BYTES_PER_GB: f64 = 1_073_741_824.0;
        write!(
            f,
            "{:.1} GB / {:.1} GB",
            self.used_bytes as f64 / BYTES_PER_GB,
            self.total_bytes as f64 / BYTES_PER_GB
        )
    }
}

pub struct SystemMetrics {
    pub cpu_usage: f32,
    pub memory: MemoryUsage,
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

        SystemMetrics {
            cpu_usage: self.system.global_cpu_usage(),
            memory: MemoryUsage {
                used_bytes: self.system.used_memory(),
                total_bytes: self.system.total_memory(),
            },
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

        assert!((0.0..=100.0).contains(&metrics.cpu_usage));
        assert!(metrics.memory.total_bytes > 0);
    }

    #[test]
    fn memory_usage_display_formats_as_gigabytes() {
        let usage = MemoryUsage {
            used_bytes: 1_073_741_824,
            total_bytes: 2_147_483_648,
        };

        assert_eq!(usage.to_string(), "1.0 GB / 2.0 GB");
    }
}
