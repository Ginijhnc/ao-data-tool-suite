//! Performance profiling utilities for CPU and memory monitoring.
//!
//! Provides background monitoring of process resource usage.

#![allow(
    clippy::std_instead_of_alloc,
    clippy::std_instead_of_core,
    reason = "std types required for tokio and sysinfo interop"
)]

use std::sync::{Arc, Mutex};
use std::time::Duration;

use sysinfo::{ProcessRefreshKind, RefreshKind, System};

pub type PeakCpu = Arc<Mutex<f32>>;
pub type PeakMemory = Arc<Mutex<f64>>;
pub type MonitorHandle = Option<tokio::task::JoinHandle<()>>;

/// Starts background CPU and memory monitoring, returning peak values.
#[must_use]
#[allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::cast_precision_loss,
    clippy::as_conversions,
    reason = "expect/unwrap acceptable for internal monitoring, precision loss acceptable for metrics"
)]
pub fn start_cpu_monitoring() -> (PeakCpu, PeakMemory, MonitorHandle) {
    let peak_cpu = Arc::new(Mutex::new(0.0_f32));
    let peak_memory = Arc::new(Mutex::new(0.0_f64));

    let peak_cpu_clone = Arc::clone(&peak_cpu);
    let peak_memory_clone = Arc::clone(&peak_memory);

    let handle = tokio::spawn(async move {
        let mut system = System::new_with_specifics(
            RefreshKind::nothing()
                .with_processes(ProcessRefreshKind::everything()),
        );
        let pid =
            sysinfo::get_current_pid().expect("Failed to get current PID");

        system.refresh_processes_specifics(
            sysinfo::ProcessesToUpdate::Some(&[pid]),
            true,
            ProcessRefreshKind::everything(),
        );

        tokio::time::sleep(Duration::from_millis(200)).await;

        loop {
            system.refresh_processes_specifics(
                sysinfo::ProcessesToUpdate::Some(&[pid]),
                true,
                ProcessRefreshKind::everything(),
            );

            if let Some(process) = system.process(pid) {
                let cpu = process.cpu_usage();
                let mem_mb = process.memory() as f64 / 1024.0 / 1024.0;

                let mut peak_cpu_lock = peak_cpu_clone.lock().unwrap();
                if cpu > *peak_cpu_lock {
                    *peak_cpu_lock = cpu;
                }
                drop(peak_cpu_lock);

                let mut peak_mem_lock = peak_memory_clone.lock().unwrap();
                if mem_mb > *peak_mem_lock {
                    *peak_mem_lock = mem_mb;
                }
                drop(peak_mem_lock);
            }

            tokio::time::sleep(Duration::from_millis(200)).await;
        }
    });

    (peak_cpu, peak_memory, Some(handle))
}
