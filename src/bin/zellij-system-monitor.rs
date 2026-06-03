use std::sync::{Arc, mpsc};
use std::thread;

use gfxinfo::active_gpu;
use glob::glob;
use single_instance::SingleInstance;

use tokio::process::Command;
use tokio::sync::mpsc as async_mpsc;
use tokio::time::{Duration, MissedTickBehavior, interval, sleep};

use zellij_load::system_info::{CpuUsage, GPU, GpuUsage, MemUsage, SystemMessage};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Check if another instance is already running
    let instance = SingleInstance::new("zellij-system-monitor").unwrap();
    if !instance.is_single() {
        eprintln!("Another instance of zellij-system-monitor is already running");
        std::process::exit(1);
    }

    println!("System Daemon started!");

    // Create channel for sending system updates
    let (tx, mut rx) = async_mpsc::channel(100);
    let tx = Arc::new(tx);

    // Create a shared system data holder
    let mut cpu_usage = CpuUsage::default();
    let mut mem_usage = MemUsage::default();
    let mut gpu_usage = GpuUsage::default();

    // Create a channel for GPU data communication
    let (gpu_tx, gpu_rx) = mpsc::channel();

    // Spawn a separate thread for GPU data collection
    thread::spawn(move || {
        loop {
            if let Ok(sys_gpu) = active_gpu() {
                let info = sys_gpu.info();
                let gpu_data = (
                    format!("{} {}", sys_gpu.vendor(), sys_gpu.model()),
                    info.used_vram(),
                    info.total_vram(),
                    info.load_pct(),
                );
                if let Err(_) = gpu_tx.send(gpu_data) {
                    break; // Channel was closed, exit thread
                }
            }
            // Sleep for 2 seconds before next update
            std::thread::sleep(Duration::from_secs(2));
        }
    });

    // Spawn a task to continuously collect system information
    let tx_clone = tx.clone();
    tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(2));
        interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
        let mut sys = sysinfo::System::new_all();

        loop {
            interval.tick().await;

            // The Zellij WASM plugin creates the lock at "/tmp/system-monitor-lock"
            // inside its sandbox, which maps to different host paths depending on
            // the system (e.g. /tmp/zellij-<uid>/ or $XDG_RUNTIME_DIR/zellij*/,
            // or just /tmp/ on some setups). Check all common locations.
            let base_tmp = std::env::var("TMPDIR").unwrap_or_else(|_| "/tmp".to_string());
            let xdg_runtime = std::env::var("XDG_RUNTIME_DIR").unwrap_or_default();
            let mut patterns: Vec<String> = vec![
                format!("{}/zellij-*/system-monitor-lock", base_tmp),
                format!("{}/system-monitor-lock", base_tmp),
            ];
            if !xdg_runtime.is_empty() {
                patterns.push(format!("{}/zellij*/system-monitor-lock", xdg_runtime));
                patterns.push(format!("{}/system-monitor-lock", xdg_runtime));
            }
            // Also cover the hard-coded /tmp fallback in case TMPDIR differs
            if base_tmp != "/tmp" {
                patterns.push("/tmp/zellij-*/system-monitor-lock".to_string());
                patterns.push("/tmp/system-monitor-lock".to_string());
            }
            let lock_exists = patterns.iter().any(|pattern| {
                match glob(pattern) {
                    Ok(mut paths) => paths.next().is_some(),
                    Err(err) => {
                        eprintln!("Error checking lock file pattern {}: {}", pattern, err);
                        false
                    }
                }
            });

            if !lock_exists {
                std::process::exit(0);
            }

            // Update CPU (two samples separated by a sleep for accurate measurement)
            cpu_usage.sample(&mut sys);
            sleep(Duration::from_millis(400)).await;
            cpu_usage.read(&mut sys);
            mem_usage.update(&mut sys);

            // Update GPU usage from the channel
            if let Ok((name, used, total, util)) = gpu_rx.try_recv() {
                gpu_usage.name = name;
                gpu_usage.memory_used = used;
                gpu_usage.memory_total = total;
                gpu_usage.gpu_utilization = util;
            }

            // Send the updated data to all clients
            let cpu_val = cpu_usage.total;
            let mem_used = mem_usage.total - mem_usage.idle;
            let mem_total = mem_usage.total;

            let gpu_info = if !gpu_usage.name.is_empty() && gpu_usage.memory_total > 0 {
                Some(GPU {
                    name: gpu_usage.name.clone(),
                    memory_used: gpu_usage.memory_used,
                    memory_total: gpu_usage.memory_total,
                    gpu_utilization: gpu_usage.gpu_utilization,
                })
            } else {
                None
            };

            let msg = SystemMessage {
                cpu_usage: cpu_val,
                mem_used,
                mem_total,
                gpu_info,
            };

            // Serialize and send the message
            if let Ok(serialized) = serde_json::to_string(&msg) {
                if let Err(e) = tx_clone.send(serialized).await {
                    eprintln!("Failed to send system update: {}", e);
                }
            }
        }
    });

    // Send system updates to this client
    while let Some(msg) = rx.recv().await {
        let _ = Command::new("zellij")
            .arg("pipe")
            .arg("--name")
            .arg("zellij-system-monitor")
            .arg("--")
            .arg(msg)
            .output()
            .await;
    }

    Ok(())
}
