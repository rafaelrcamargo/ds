use clap::ArgMatches;
use colored::{ColoredString, Colorize};
use terminal_size::{terminal_size, Width};

/// Builds the command to run.
pub fn build_command(matches: ArgMatches) -> Vec<String> {
    let mut command = vec!["stats", "--format", "json"];

    matches
        .get_many::<String>("CONTAINER")
        .into_iter()
        .flatten()
        .for_each(|c| command.push(c));

    command.iter().map(|s| s.to_string()).collect()
}

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

/// Fills the size with the given char, but only on even numbers.
pub fn fill_on_even(char: &str, size: usize, len: usize) -> String {
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

/// Parses a percentage string into a usize.
pub fn perc_to_usize(perc: String) -> usize {
    if perc.contains('%') {
        perc.replace('%', "")
            .parse::<f32>()
            .expect("Failed to parse Percentage")
            .round() as usize
    } else {
        0
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
pub fn balanced_split(value: u128) -> Vec<u128> {
    vec![(value / 2) as usize, (value / 2 + value % 2) as usize]
        .into_iter()
        .map(|v| v as u128)
        .collect()
}

/// Scales a vector of numbers between a min and max value.
pub fn scale_between(
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
