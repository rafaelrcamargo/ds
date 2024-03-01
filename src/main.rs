mod cli;
mod data;
mod utils;

use byte_unit::Byte;
use colored::Colorize;
use data::DockerStats;
use std::{
    io::{BufRead, BufReader},
    process::{Command, Stdio},
    sync::{
        mpsc::{self, Receiver},
        Arc, Mutex
    },
    thread,
    time::Instant
};
use utils::*;

fn main() {
    let matches = cli::args().get_matches();
    let (compact, full) = (get_flag(&matches, "compact"), get_flag(&matches, "full"));

    let containers: Arc<Mutex<Vec<DockerStats>>> = Arc::new(Mutex::new(Vec::new()));
    let width = get_terminal_width();

    let (sender, receiver) = mpsc::channel::<()>();

    let t_containers = containers.clone();
    let purger = thread::spawn(move || purge(receiver, t_containers, compact, full, width));

    let mut cmd = Command::new("docker")
        .args(build_command(matches))
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to run \"docker stats ...\"");

    let stdout = cmd.stdout.as_mut().unwrap();
    let stdout_reader = BufReader::new(stdout);
    let stdout_lines = stdout_reader.lines();

    // print!("\x1B[s"); // Save cursor position

    // Clear line and print
    // fn cap(string: String) { print!("\x1B[K{}\n", string) }

    println!("Fetching container stats...");

    for line in stdout_lines {
        let mut line = line.unwrap();
        // dbg!(line.clone());

        sender.send(()).unwrap();

        let containers = containers.clone();
        if line.starts_with("\u{1b}[2J\u{1b}[H") {
            // println!("{}", "\x1B[u"); // Restore cursor position
            print(containers.clone(), compact, full, width); // Print the charts
            containers.lock().unwrap().clear(); // Reset the containers

            line = line.replace("\u{1b}[2J\u{1b}[H", "")
        }

        let stats: DockerStats = serde_json::from_str(&line).unwrap();
        containers.lock().unwrap().push(stats);
    }

    purger.join().unwrap();
    let status = cmd.wait();
    println!("Exited with status {:?}", status);
}

fn purge(receiver: Receiver<()>, containers: Arc<Mutex<Vec<DockerStats>>>, compact: bool, full: bool, width: usize) {
    let mut last_message: Instant = Instant::now();

    loop {
        if receiver.try_recv().is_ok() {
            last_message = Instant::now()
        }

        if last_message.elapsed().as_secs() > 2 {
            containers.lock().unwrap().clear(); // Clear the containers list
            print(containers.clone(), compact, full, width); // Print the charts
            last_message = Instant::now()
        }
    }
}

fn print(containers: Arc<Mutex<Vec<DockerStats>>>, compact: bool, full: bool, width: usize) {
    print!("\x1B[2J\x1B[1;1H"); // Clear screen

    let containers = containers.lock().unwrap();
    let mut max = 100f32;

    if containers.len() == 0 {
        println!("Waiting for container stats...");
        return;
    }

    for (i, stats) in containers.iter().enumerate() {
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
                    let net = stats.net_io.split(" / ").collect::<Vec<&str>>();

                    let bytes = vec![
                        Byte::parse_str(net[0], true).unwrap().as_u128(),
                        Byte::parse_str(net[1], true).unwrap().as_u128(),
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
                    let blocks = stats.block_io.split(" / ").collect::<Vec<&str>>();

                    let bytes = vec![
                        Byte::parse_str(blocks[0], true)
                            .expect("Failed to parse Byte")
                            .as_u128(),
                        Byte::parse_str(blocks[1], true)
                            .expect("Failed to parse Byte")
                            .as_u128(),
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

        if !compact || i == containers.len() - 1 {
            println!("└{}┘", filler("─", width, 2));
        }
    }
}
