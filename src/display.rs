use crate::{data::DockerStats, utils::*};
use byte_unit::Byte;
use colored::Colorize;
use std::io::{self, Write};

pub struct StatsDisplay {
    width: usize,
    compact: bool,
    full: bool
}

impl StatsDisplay {
    pub fn new(width: usize, compact: bool, full: bool) -> Self {
        // Hide cursor once at start
        print!("\x1B[?25l");
        let _ = io::stdout().flush();
        Self { width, compact, full }
    }

    /// Print a line after erasing the current one to avoid leftover characters
    fn out_line(&self, line: &str) {
        // 2K – erase entire line, \r – carriage return, then newline
        print!("\x1B[2K\r{}\n", line);
    }

    // Ensure cursor is shown again when the display is dropped (program exit)
}

impl Drop for StatsDisplay {
    fn drop(&mut self) {
        // Show cursor back
        print!("\x1B[?25h");
        let _ = io::stdout().flush();
    }
}

impl StatsDisplay {
    pub fn print_stats(&self, containers: &[DockerStats]) {
        // Move cursor to home (top-left) without erasing the entire screen
        print!("\x1B[H");

        let mut max = 100f32;

        if containers.is_empty() {
            self.out_line("Waiting for container stats...");
        } else {
            // Calculate global scale
            for stats in containers {
                let mem_perc = perc_to_float(&stats.mem_perc);
                let cpu_perc = perc_to_float(&stats.cpu_perc);
                max = max.max(mem_perc).max(cpu_perc);
            }

            for (i, stats) in containers.iter().enumerate() {
                self.print_container_stats(stats, i, containers.len(), max);
            }
        }

        self.out_line("Press Ctrl+C to exit");

        // Clear anything below the current cursor position (in case the new frame is shorter)
        print!("\x1B[J");

        // Flush to ensure the frame is pushed in one burst, reducing flicker
        let _ = io::stdout().flush();
    }

    fn print_container_stats(&self, stats: &DockerStats, index: usize, total: usize, max: f32) {
        // LAYOUT
        if !self.compact || index == 0 {
            self.out_line(&format!("┌─ {} {}┐", stats.name, filler("─", self.width, stats.name.len() + 5)));
        } else {
            self.out_line(&format!("├─ {} {}┤", stats.name, fill_on_even("─", self.width, stats.name.len() + 5)));
        }

        let mem_perc = perc_to_float(&stats.mem_perc);
        let cpu_perc = perc_to_float(&stats.cpu_perc);

        // CPU
        let scale_factor = (self.width - 18) as f32 / max;
        let cpu_perc_scaled = (cpu_perc * scale_factor) as usize;
        let cpu_padding = filler(" ", 7, stats.cpu_perc.len());
        let cpu_status = usize_to_status(cpu_perc_scaled, self.width);
        let cpu_fill = filler("░", self.width, cpu_perc_scaled + 18).dimmed();

        self.out_line(&format!("│ CPU | {cpu_padding}{} {cpu_status}{cpu_fill} │", stats.cpu_perc));

        // RAM
        let mem_usage_len = stats.mem_usage.len() + 1;
        let scale_factor = (self.width - (18 + mem_usage_len)) as f32 / max;
        let mem_perc_scaled = (mem_perc * scale_factor) as usize;
        let mem_padding = filler(" ", 7, stats.mem_perc.len());
        let mem_status = usize_to_status(mem_perc_scaled, self.width - (18 + mem_usage_len));
        let mem_fill = filler("░", self.width, mem_perc_scaled + (18 + mem_usage_len)).dimmed();
        let mem_spacing = filler(" ", mem_usage_len, mem_usage_len);

        self.out_line(&format!(
            "│ RAM | {mem_padding}{} {mem_status}{mem_fill}{mem_spacing} {} │",
            stats.mem_perc, stats.mem_usage
        ));

        if self.full {
            self.print_full_stats(stats);
        }

        if !self.compact || index == total - 1 {
            self.out_line(&format!("└{}┘", filler("─", self.width, 2)));
        }
    }

    fn print_full_stats(&self, stats: &DockerStats) {
        self.out_line(&format!("│{}│", fill_on_even("─", self.width, 2).dimmed()));

        // NET
        if let Ok(net) = self.parse_network_stats(&stats.net_io) {
            self.out_line(&format!(
                "│ NET | {}{}{} │",
                filler("▒", self.width - 11, net[0]).green(),
                "░".dimmed(),
                filler("▒", self.width - 11, net[1]).red()
            ));
        }

        // IO
        if let Ok(io) = self.parse_block_stats(&stats.block_io) {
            self.out_line(&format!(
                "│  IO | {}{}{} │",
                filler("▒", io[0], 0).white(),
                "░".dimmed(),
                filler("▒", io[1], 0).black()
            ));
        }
    }

    fn parse_network_stats(&self, net_io: &str) -> Result<Vec<usize>, Box<dyn std::error::Error>> {
        let parts: Vec<&str> = net_io.split(" / ").collect();
        if parts.len() != 2 {
            return Ok(balanced_split(self.width - 11));
        }

        let bytes = vec![
            Byte::parse_str(parts[0], true)?.as_u128(),
            Byte::parse_str(parts[1], true)?.as_u128(),
        ];

        Ok(scale_between(bytes, 1, self.width - 12).unwrap_or_else(|| balanced_split(self.width - 11)))
    }

    fn parse_block_stats(&self, block_io: &str) -> Result<Vec<usize>, Box<dyn std::error::Error>> {
        let parts: Vec<&str> = block_io.split(" / ").collect();
        if parts.len() != 2 {
            return Ok(balanced_split(self.width - 11));
        }

        let bytes = vec![
            Byte::parse_str(parts[0], true)?.as_u128(),
            Byte::parse_str(parts[1], true)?.as_u128(),
        ];

        Ok(scale_between(bytes, 1, self.width - 12).unwrap_or_else(|| balanced_split(self.width - 11)))
    }
}
