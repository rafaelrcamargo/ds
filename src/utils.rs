#![allow(dead_code)]

use clap::ArgMatches;
use colored::{ColoredString, Colorize};
use terminal_size::{terminal_size, Width};

/// Builds the command to run.
pub fn build_command(matches: ArgMatches) -> Vec<String> {
    let mut command = vec!["stats".to_string(), "--format".to_string(), "json".to_string()];

    if let Some(containers) = matches.get_many::<String>("CONTAINER") {
        command.extend(containers.cloned());
    }

    command
}

pub fn get_flag(args: &ArgMatches, id: &str) -> bool { args.get_one::<bool>(id).is_some_and(|x| *x) }

/// Gets the current terminal width.
pub fn get_terminal_width() -> usize {
    if let Some((Width(w), _)) = terminal_size() {
        w.into()
    } else {
        80
    }
}

/// Fills the size with the given char.
pub fn filler(char: &str, max: usize, used: usize) -> String {
    if max == 0 || max <= used {
        String::new()
    } else {
        char.repeat(max - used)
    }
}

/// Fills the size with the given char, but only on even numbers.
pub fn fill_on_even(char: &str, size: usize, len: usize) -> String {
    if size == 0 || size <= len {
        String::new()
    } else {
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
}

/// Parses a percentage string into a usize.
pub fn perc_to_float(perc: &str) -> f32 {
    if let Some(stripped) = perc.strip_suffix('%') {
        stripped.parse::<f32>().unwrap_or(0.0)
    } else {
        0.0
    }
}

/// Converts a usize to a colored status bar.
pub fn usize_to_status(perc: usize, max: usize) -> ColoredString {
    let fill = filler("█", perc, 0);

    if perc < max / 2 {
        fill.green()
    } else if perc < max - (max / 4) {
        fill.yellow()
    } else {
        fill.red()
    }
}

/// Splits a value into two balanced parts.
pub fn balanced_split(value: usize) -> Vec<usize> { vec![value / 2, value / 2 + value % 2] }

/// Scales a vector of numbers between a min and max value.
pub fn scale_between(nums: Vec<u128>, floor: usize, ceil: usize) -> Option<Vec<usize>> {
    let min = *nums.iter().min()?;
    let max = *nums.iter().max()?;
    let [floor, ceil] = [floor as u128, ceil as u128];

    if min == max {
        return None;
    }

    let scaled = nums
        .iter()
        .map(|num| {
            let scaled_val = (ceil - floor) * (num - min) / (max - min) + floor;
            usize::try_from(scaled_val).unwrap_or(0)
        })
        .collect();

    Some(scaled)
}
