use byte_unit::Byte;
use colored::{ColoredString, Colorize};
use serde::Deserialize;
use std::{
    collections::HashMap,
    io::{BufRead, BufReader},
    process::{Command, Stdio},
};
use terminal_size::{terminal_size, Width};

fn scale_between(
    unscaled_nums: Vec<u128>,
    min_allowed: u128,
    max_allowed: u128,
) -> Option<Vec<u128>> {
    let min = unscaled_nums.iter().min().unwrap();
    let max = unscaled_nums.iter().max().unwrap();

    if min == max {
        return None;
    }

    let scaled_nums: Vec<_> = unscaled_nums
        .iter()
        .map(|num| (max_allowed - min_allowed) * (num - min) / (max - min) + min_allowed)
        .collect();

    Some(scaled_nums)
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct DockerStats {
    #[serde(rename = "BlockIO")]
    block_io: String,
    #[serde(rename = "CPUPerc")]
    cpu_perc: String,
    #[serde(rename = "ID")]
    id: String,
    mem_perc: String,
    mem_usage: String,
    name: String,
    #[serde(rename = "NetIO")]
    net_io: String,
}

fn main() {
    let mut containers: HashMap<String, DockerStats> = HashMap::new();
    let width = get_terminal_width();

    let mut cmd = Command::new("docker")
        .args(["stats", "--format", "json"])
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

        let mut max = 100_f32;

        for (_, (_, stats)) in containers.iter().enumerate() {
            //if i == 0 {
            println!(
                "┌─ {} {}┐",
                stats.name,
                filler("─", width, stats.name.len() + 5)
            );
            /* } else {
                println!(
                    "├─ {} {}┤",
                    stats.name,
                    fill_on_even("─", width, stats.name.len() + 5)
                );
            } */

            let mem_perc = perc_to_usize(stats.mem_perc.clone());
            let cpu_perc = perc_to_usize(stats.cpu_perc.clone());

            max = max.max(mem_perc as f32);
            max = max.max(cpu_perc as f32);

            // CPU
            let scale_factor = (width - 18) as f32 / max;
            let cpu_perc = (cpu_perc as f32 * scale_factor) as usize;
            println!(
                "│ CPU | {}{} {}{} │",
                filler(" ", 7, stats.cpu_perc.len()),
                stats.cpu_perc,
                usize_to_status(cpu_perc, width),
                filler("░", width, cpu_perc + 18).dimmed()
            );

            // RAM
            let mem_usage_len = stats.mem_usage.len() + 1;
            let scale_factor = (width - (18 + mem_usage_len)) as f32 / max;
            let mem_perc = (mem_perc as f32 * scale_factor) as usize;
            println!(
                "│ RAM | {}{} {}{}{} {} │",
                filler(" ", 7, stats.mem_perc.len()),
                stats.mem_perc,
                usize_to_status(mem_perc, width - (18 + mem_usage_len)),
                filler("░", width, mem_perc + (18 + mem_usage_len)).dimmed(),
                filler(" ", mem_usage_len, mem_usage_len),
                stats.mem_usage
            );

            println!("│{}│", fill_on_even("─", width, 2).dimmed());

            // NET

            let net = stats.net_io.split(" / ").collect::<Vec<&str>>();

            let inp = Byte::from_str(net[0])
                .expect("Failed to parse Byte")
                .get_bytes();

            let out = Byte::from_str(net[1])
                .expect("Failed to parse Byte")
                .get_bytes();

            let scaled_net = match scale_between(vec![inp, out], 1, (width - 12) as u128) {
                None => full_split(width as u128 - 11),
                Some(scaled_net) => scaled_net,
            };

            println!(
                "│ NET | {}{}{} │",
                filler("▒", scaled_net[0] as usize, width - 12).green(),
                "░".dimmed(),
                filler("▒", scaled_net[1] as usize, width - 12).red()
            );

            // IO

            let blocks = stats.block_io.split(" / ").collect::<Vec<&str>>();

            let inp = Byte::from_str(blocks[0])
                .expect("Failed to parse Byte")
                .get_bytes();
            let out = Byte::from_str(blocks[1])
                .expect("Failed to parse Byte")
                .get_bytes();

            let scaled_io = match scale_between(vec![inp, out], 1, (width - 12) as u128) {
                None => full_split(width as u128 - 11),
                Some(scaled_io) => scaled_io,
            };

            println!(
                "│  IO | {}{}{} │",
                filler("▒", scaled_io[0] as usize, 0).white(),
                "░".dimmed(),
                filler("▒", scaled_io[1] as usize, 0).black()
            );

            //if i == containers.len() - 1 {
            println!("└{}┘", filler("─", width, 2));
            //}
        }
    }

    let status = cmd.wait();
    println!("Exited with status {:?}", status);
}

fn get_terminal_width() -> usize {
    if let Some((Width(w), _)) = terminal_size() {
        w.into()
    } else {
        80
    }
}

fn filler(char: &str, max: usize, used: usize) -> String {
    if max == 0 {
        String::new()
    } else if max < used {
        // Not really sure when this would happen,
        // and neither if this is the correct way to handle it...
        char.repeat(max)
    } else {
        char.repeat(max - used)
    }
}

fn fill_on_even(char: &str, size: usize, len: usize) -> String {
    let mut filler = String::new();

    for i in 0..(size - len) {
        if i % 2 == 0 {
            filler.push_str(char);
        } else {
            filler.push(' ');
        }
    }

    filler
}

fn perc_to_usize(perc: String) -> usize {
    if perc.contains('%') {
        perc.replace('%', "")
            .parse::<f32>()
            .expect("Failed to parse Percentage")
            .round() as usize
    } else {
        0
    }
}

fn usize_to_status(perc: usize, max: usize) -> ColoredString {
    let fill = filler("█", perc, 0);

    if perc < max / 2 {
        fill.green()
    } else if perc < max - (max / 4) {
        fill.yellow()
    } else {
        fill.red()
    }
}

fn full_split(value: u128) -> Vec<u128> {
    vec![(value / 2) as usize, (value / 2 + value % 2) as usize]
        .into_iter()
        .map(|v| v as u128)
        .collect()
}
