use std::fmt;
use sysinfo::{CpuRefreshKind, MemoryRefreshKind, RefreshKind, System};

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
    /// `None` until two samples at least `sysinfo::MINIMUM_CPU_UPDATE_INTERVAL`
    /// apart have been taken; sysinfo needs that pair to compute a meaningful
    /// usage delta, so the very first sample is otherwise meaningless rather
    /// than genuinely 0%.
    pub cpu_usage: Option<f32>,
    pub memory: MemoryUsage,
}

pub struct MetricsCollector {
    system: System,
    has_cpu_baseline: bool,
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricsCollector {
    pub fn new() -> Self {
        let refresh_kind = RefreshKind::nothing()
            .with_cpu(CpuRefreshKind::nothing().with_cpu_usage())
            .with_memory(MemoryRefreshKind::nothing().with_ram());
        let system = System::new_with_specifics(refresh_kind);
        Self {
            system,
            has_cpu_baseline: false,
        }
    }

    pub fn fetch(&mut self) -> SystemMetrics {
        self.system.refresh_cpu_usage();
        self.system.refresh_memory();

        let cpu_usage = if self.has_cpu_baseline {
            Some(self.system.global_cpu_usage())
        } else {
            self.has_cpu_baseline = true;
            None
        };

        SystemMetrics {
            cpu_usage,
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
    fn first_fetch_reports_cpu_unavailable() {
        let mut collector = MetricsCollector::new();
        let metrics = collector.fetch();

        assert_eq!(metrics.cpu_usage, None);
        assert!(metrics.memory.total_bytes > 0);
    }

    #[test]
    fn second_fetch_after_minimum_interval_reports_cpu_usage() {
        let mut collector = MetricsCollector::new();
        collector.fetch();
        std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
        let metrics = collector.fetch();

        assert!(
            metrics
                .cpu_usage
                .is_some_and(|cpu| (0.0..=100.0).contains(&cpu))
        );
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
