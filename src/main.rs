mod cli;
mod data;
mod utils;

use byte_unit::Byte;
use colored::Colorize;
use data::DockerStats;
use std::{process::Stdio, sync::Arc, time::Duration};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Command,
    sync::{mpsc, Mutex},
    task, time
};
use utils::*;

#[tokio::main]
async fn main() {
    let matches = cli::args().get_matches();
    let (compact, full) = (
        // Get the args
        cli::has_arg(&matches, "compact"),
        cli::has_arg(&matches, "full")
    );

    let containers: Arc<Mutex<Vec<DockerStats>>> = Arc::new(Mutex::new(Vec::new()));
    let width = get_terminal_width();

    let (debounce_tx, mut debounce_rx) = mpsc::channel(1);

    // Listen for events
    let debouncer = task::spawn({
        let (compact, full, width) = (compact.clone(), full.clone(), width.clone());
        let containers_ref = containers.clone();

        async move {
            let duration = Duration::from_secs(2);

            loop {
                match time::timeout(duration, debounce_rx.recv()).await {
                    Ok(Some(())) => (),
                    Ok(None) => break,
                    Err(_) => {
                        print(&containers_ref, compact, full, width).await; // Print the charts
                        containers_ref.lock().await.clear(); // Reset the containers

                        println!("No events for {}s, waiting...", duration.as_secs())
                    }
                }
            }
        }
    });

    let command = task::spawn({
        let debounce_tx = debounce_tx.clone();

        async move {
            let mut cmd = Command::new("docker")
                .args(build_command(matches))
                .stdout(Stdio::piped())
                .spawn()
                .expect("Failed to run \"docker stats ...\"");

            let stdout = cmd.stdout.take().unwrap();
            let mut stdout_reader = BufReader::new(stdout).lines();

            // print!("\x1B[s"); // Save cursor position

            // Clear line and print
            // fn cap(string: String) { print!("\x1B[K{}\n", string) }

            println!("Fetching container stats...");

            while let Some(line) = stdout_reader.next_line().await.unwrap() {
                // Send a ping to the debouncer
                debounce_tx.send(()).await.unwrap();

                if line.starts_with("\u{1b}[2J\u{1b}[H") && !containers.lock().await.is_empty() {
                    // println!("{}", "\x1B[u"); // Restore cursor position
                    print(&containers, compact, full, width).await; // Print the charts
                    containers.lock().await.clear(); // Reset the containers
                }

                let line = line.replace("\u{1b}[2J\u{1b}[H", "");

                let stats: DockerStats = serde_json::from_str(&line).unwrap();
                containers.lock().await.push(stats);
            }

            let status = cmd.wait().await.unwrap();
            println!("Exited with status: {}", status);
        }
    });

    command.await.expect("Error on cmd.");
    debouncer.await.expect("Error on deb.");
}

async fn print(containers: &Arc<Mutex<Vec<DockerStats>>>, compact: bool, full: bool, width: usize) {
    print!("\x1B[2J\x1B[1;1H"); // Clear screen

    let cont = containers.lock().await;
    let mut max = 100f32;

    for (i, stats) in cont.iter().enumerate() {
        // LAYOUT
        {
            if !compact || i == 0 {
                println!("┌─ {} {}┐", stats.name, filler("─", width, stats.name.len() + 5));
            } else {
                println!("├─ {} {}┤", stats.name, fill_on_even("─", width, stats.name.len() + 5));
            }
        }

        // CONSTANTS
        let mem_perc: f32 = perc_to_float(stats.mem_perc.clone());
        let cpu_perc: f32 = perc_to_float(stats.cpu_perc.clone());

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
                    let net = stats
                        .net_io
                        .split(" / ")
                        .collect::<Vec<&str>>();

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
                        Some(scaled_net) => scaled_net
                    }
                };

                println!(
                    "│ NET | {}{}{} │",
                    filler("▒", width - 11, net[0]).green(),
                    "░".dimmed(),
                    filler("▒", width - 11, net[1]).red()
                );
            }

            // IO
            {
                let io = {
                    let blocks = stats
                        .block_io
                        .split(" / ")
                        .collect::<Vec<&str>>();

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
                        Some(scaled_io) => scaled_io
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

        if !compact || i == cont.len() - 1 {
            println!("└{}┘", filler("─", width, 2));
        }
    }
}
