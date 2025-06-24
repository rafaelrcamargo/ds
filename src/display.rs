use crate::{data::DockerStats, utils::*};
use byte_unit::Byte;
use colored::Colorize;

pub struct StatsDisplay {
    width: usize,
    compact: bool,
    full: bool
}

impl StatsDisplay {
    pub fn new(width: usize, compact: bool, full: bool) -> Self { Self { width, compact, full } }

    pub fn print_stats(&self, containers: &[DockerStats]) {
        print!("\x1B[2J\x1B[1;1H"); // Clear screen

        if containers.is_empty() {
            println!("Waiting for container stats...");
            println!("Press Ctrl+C to exit");
            return;
        }

        let mut max = 100f32;

        // Calculate global scale
        for stats in containers {
            let mem_perc = perc_to_float(&stats.mem_perc);
            let cpu_perc = perc_to_float(&stats.cpu_perc);
            max = max.max(mem_perc).max(cpu_perc);
        }

        for (i, stats) in containers.iter().enumerate() {
            self.print_container_stats(stats, i, containers.len(), max);
        }

        println!("Press Ctrl+C to exit");
    }

    fn print_container_stats(&self, stats: &DockerStats, index: usize, total: usize, max: f32) {
        // LAYOUT
        if !self.compact || index == 0 {
            println!("┌─ {} {}┐", stats.name, filler("─", self.width, stats.name.len() + 5));
        } else {
            println!("├─ {} {}┤", stats.name, fill_on_even("─", self.width, stats.name.len() + 5));
        }

        let mem_perc = perc_to_float(&stats.mem_perc);
        let cpu_perc = perc_to_float(&stats.cpu_perc);

        // CPU
        let scale_factor = (self.width - 18) as f32 / max;
        let cpu_perc_scaled = (cpu_perc * scale_factor) as usize;
        let cpu_padding = filler(" ", 7, stats.cpu_perc.len());
        let cpu_status = usize_to_status(cpu_perc_scaled, self.width);
        let cpu_fill = filler("░", self.width, cpu_perc_scaled + 18).dimmed();

        println!("│ CPU | {cpu_padding}{} {cpu_status}{cpu_fill} │", stats.cpu_perc);

        // RAM
        let mem_usage_len = stats.mem_usage.len() + 1;
        let scale_factor = (self.width - (18 + mem_usage_len)) as f32 / max;
        let mem_perc_scaled = (mem_perc * scale_factor) as usize;
        let mem_padding = filler(" ", 7, stats.mem_perc.len());
        let mem_status = usize_to_status(mem_perc_scaled, self.width - (18 + mem_usage_len));
        let mem_fill = filler("░", self.width, mem_perc_scaled + (18 + mem_usage_len)).dimmed();
        let mem_spacing = filler(" ", mem_usage_len, mem_usage_len);

        println!(
            "│ RAM | {mem_padding}{} {mem_status}{mem_fill}{mem_spacing} {} │",
            stats.mem_perc, stats.mem_usage
        );

        if self.full {
            self.print_full_stats(stats);
        }

        if !self.compact || index == total - 1 {
            println!("└{}┘", filler("─", self.width, 2));
        }
    }

    fn print_full_stats(&self, stats: &DockerStats) {
        println!("│{}│", fill_on_even("─", self.width, 2).dimmed());

        // NET
        if let Ok(net) = self.parse_network_stats(&stats.net_io) {
            println!(
                "│ NET | {}{}{} │",
                filler("▒", self.width - 11, net[0]).green(),
                "░".dimmed(),
                filler("▒", self.width - 11, net[1]).red()
            );
        }

        // IO
        if let Ok(io) = self.parse_block_stats(&stats.block_io) {
            println!(
                "│  IO | {}{}{} │",
                filler("▒", io[0], 0).white(),
                "░".dimmed(),
                filler("▒", io[1], 0).black()
            );
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
