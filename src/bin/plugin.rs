use std::collections::BTreeMap;
use std::fs::{self, File};

use colored::Colorize;

use zellij_load::system_info::SystemMessage;
use zellij_tile::prelude::*;

#[derive(Default)]
struct State {
    stats: SystemMessage,
}

register_plugin!(State);

fn strip_ansi_codes(s: &str) -> String {
    // Remove ANSI escape sequences
    let re = regex::Regex::new(r"\x1b\[[0-9;]*[mK]").unwrap();
    re.replace_all(s, "").to_string()
}

impl ZellijPlugin for State {
    fn load(&mut self, _configuration: BTreeMap<String, String>) {
        // Request the necessary permissions
        request_permission(&[PermissionType::RunCommands, PermissionType::OpenFiles]);

        subscribe(&[
            EventType::PermissionRequestResult,
            EventType::RunCommandResult,
            EventType::BeforeClose,
        ]);
    }

    fn update(&mut self, event: Event) -> bool {
        let should_render = false;

        match event {
            Event::PermissionRequestResult(status) => match status {
                PermissionStatus::Granted => {
                    eprintln!("Permission granted");
                    let _ = File::create("/tmp/system-monitor-lock").unwrap();
                    // Get current directory and run command there
                    let current_dir = std::path::PathBuf::from(".");

                    // Pass plugin PID to monitor using environment variable
                    run_command_with_env_variables_and_cwd(
                        &["zellij_system_monitor"],
                        BTreeMap::new(),
                        current_dir,
                        BTreeMap::new(),
                    );
                }
                PermissionStatus::Denied => {
                    eprintln!("Permission denied");
                }
            },
            Event::RunCommandResult(_exit_code, out, error, _context) => {
                eprintln!(
                    "Command Ran {} {:?}",
                    String::from_utf8_lossy(&out),
                    String::from_utf8_lossy(&error)
                );
            }
            Event::BeforeClose => {
                eprintln!("Before close event received");
                let _ = fs::remove_file("/tmp/system-monitor-lock");
            }
            _ => {}
        }

        should_render
    }

    fn pipe(&mut self, pipe_message: PipeMessage) -> bool {
        let mut should_render = false;
        if let PipeSource::Cli(_input_pipe_id) = pipe_message.source
            && let Some(payload) = pipe_message.payload
        {
            match serde_json::from_str(&payload) as Result<SystemMessage, _> {
                // Deserialize the JSON message
                Ok(system_msg) => {
                    self.stats = system_msg;
                    should_render = true;
                }
                Err(e) => {
                    eprintln!("Failed to parse message: {}", e);
                }
            }
        }
        should_render
    }

    fn render(&mut self, _rows: usize, cols: usize) {
        if cols == 0 {
            return;
        }

        let mut segments: Vec<String> = Vec::new();

        let cpu_value = format!("{:.2}%", self.stats.cpu_usage as f64);
        let cpu_segment = format!("CPU: {}", cpu_value.magenta());
        segments.push(cpu_segment);

        let mem_used_gb = self.stats.mem_used as f64 / 1024.0 / 1024.0 / 1024.0;
        let mem_total_gb = self.stats.mem_total as f64 / 1024.0 / 1024.0 / 1024.0;
        let mem_value = format!("{:.2}GB/{:.2}GB", mem_used_gb, mem_total_gb);
        let mem_segment = format!("MEM: {}", mem_value.blue());
        segments.push(mem_segment);

        if let Some(gpu) = &self.stats.gpu_info {
            let gpu_value = format!("{:.2}%", gpu.gpu_utilization as f64);
            let gpu_segment = format!("GPU: {}", gpu_value.green());
            segments.push(gpu_segment);

            let vram_used_gb = gpu.memory_used as f64 / 1024.0 / 1024.0 / 1024.0;
            let vram_total_gb = gpu.memory_total as f64 / 1024.0 / 1024.0 / 1024.0;
            let vram_value = format!("{:.2}GB/{:.2}GB", vram_used_gb, vram_total_gb);
            let vram_segment = format!("VRAM: {}", vram_value.yellow());
            segments.push(vram_segment);
        }

        let mut fitted: Vec<String> = Vec::new();
        let mut used_cols = 0;

        for segment in segments {
            let segment_visible = strip_ansi_codes(&segment).chars().count();
            let space_needed = if fitted.is_empty() {
                segment_visible
            } else {
                3 + segment_visible // 3 = length of separator " | "
            };

            if used_cols + space_needed <= cols {
                used_cols += space_needed;
                fitted.push(segment);
            } else {
                break;
            }
        }

        if fitted.is_empty() {
            return;
        }

        let text = fitted.join(" | ");
        let visible_length = strip_ansi_codes(&text).chars().count();
        let padding = cols.saturating_sub(visible_length);
        print!("{}{}", " ".repeat(padding), text);
    }
}
