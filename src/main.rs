mod cli;

mod utils;
use utils::*;

mod data;
use data::DockerStats;

use byte_unit::Byte;
use colored::Colorize;
use std::{
    collections::HashMap,
    io::{BufRead, BufReader},
    process::{Command, Stdio},
};

fn main() {
    let matches = cli::args().get_matches();
    let (compact, full) = (
        // Get the args
        cli::has_arg(&matches, "compact"),
        cli::has_arg(&matches, "full"),
    );

    let mut containers: HashMap<String, DockerStats> = HashMap::new();
    let width = get_terminal_width();

    let mut cmd = Command::new("docker")
        .args(build_command(matches))
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to run \"docker stats ...\"");

    let stdout = cmd.stdout.as_mut().unwrap();
    let stdout_reader = BufReader::new(stdout);
    let stdout_lines = stdout_reader.lines();

    for line in stdout_lines {
        print!("\x1B[2J\x1B[1;1H"); // Clear screen

        let line = line.unwrap().replace("\u{1b}[2J\u{1b}[H", "");
        let stats: DockerStats = serde_json::from_str(&line).unwrap();
        containers.insert(stats.id.clone(), stats);

        let mut max = 100f32;

        for (i, (_id, stats)) in containers.iter().enumerate() {
            // LAYOUT
            {
                if !compact || i == 0 {
                    println!(
                        "┌─ {} {}┐",
                        stats.name,
                        filler("─", width, stats.name.len() + 5)
                    );
                } else {
                    println!(
                        "├─ {} {}┤",
                        stats.name,
                        fill_on_even("─", width, stats.name.len() + 5)
                    );
                }
            }

            // CONSTANTS
            let mem_perc: f32 = perc_to(stats.mem_perc.clone());
            let cpu_perc: f32 = perc_to(stats.cpu_perc.clone());

            // GLOBAL SCALE
            max = max.max(mem_perc);
            max = max.max(cpu_perc);

            // CPU
            {
                let scale_factor = (width - 18) as f32 / max;
                let cpu_perc = (cpu_perc * scale_factor) as usize;
                println!(
                    "│ CPU | {}{} {}{} │",
                    filler(" ", 7, stats.cpu_perc.len()),
                    stats.cpu_perc,
                    usize_to_status(cpu_perc, width),
                    filler("░", width, cpu_perc + 18).dimmed()
                );
            }

            // RAM
            {
                let mem_usage_len = stats.mem_usage.len() + 1;
                let scale_factor = (width - (18 + mem_usage_len)) as f32 / max;
                let mem_perc = (mem_perc * scale_factor) as usize;
                println!(
                    "│ RAM | {}{} {}{}{} {} │",
                    filler(" ", 7, stats.mem_perc.len()),
                    stats.mem_perc,
                    usize_to_status(mem_perc, width - (18 + mem_usage_len)),
                    filler("░", width, mem_perc + (18 + mem_usage_len)).dimmed(),
                    filler(" ", mem_usage_len, mem_usage_len),
                    stats.mem_usage
                );
            }

            if full {
                println!("│{}│", fill_on_even("─", width, 2).dimmed());

                // NET
                {
                    let net: Vec<usize> = {
                        let net = stats.net_io.split(" / ").collect::<Vec<&str>>();

                        let bytes = vec![
                            Byte::from_str(net[0])
                                .expect("Failed to parse Byte")
                                .get_bytes(),
                            Byte::from_str(net[1])
                                .expect("Failed to parse Byte")
                                .get_bytes(),
                        ];

                        match scale_between(bytes, 1, width - 12) {
                            None => balanced_split(width - 11),
                            Some(scaled_net) => scaled_net,
                        }
                    };

                    println!(
                        "│ NET | {}{}{} │",
                        filler("▒", net[0], width - 12).green(),
                        "░".dimmed(),
                        filler("▒", net[1], width - 12).red()
                    );
                }

                // IO
                {
                    let io = {
                        let blocks = stats.block_io.split(" / ").collect::<Vec<&str>>();

                        let bytes = vec![
                            Byte::from_str(blocks[0])
                                .expect("Failed to parse Byte")
                                .get_bytes(),
                            Byte::from_str(blocks[1])
                                .expect("Failed to parse Byte")
                                .get_bytes(),
                        ];

                        match scale_between(bytes, 1, width - 12) {
                            None => balanced_split(width - 11),
                            Some(scaled_io) => scaled_io,
                        }
                    };

                    println!(
                        "│  IO | {}{}{} │",
                        filler("▒", io[0], 0).white(),
                        "░".dimmed(),
                        filler("▒", io[1], 0).black()
                    );
                }
            }

            if !compact || i == containers.len() - 1 {
                println!("└{}┘", filler("─", width, 2));
            }
        }
    }

    let status = cmd.wait();
    println!("Exited with status {:?}", status);
}
