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

    command
        .iter()
        .map(|s| s.to_string())
        .collect()
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
pub fn perc_to<T: From<f32>>(perc: String) -> T {
    if perc.contains('%') {
        perc.replace('%', "")
            .parse::<f32>()
            .expect("Failed to parse percentage")
            .into()
    } else {
        0f32.into()
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
    let [min, max] = [nums.iter().min().unwrap(), nums.iter().max().unwrap()];
    let [floor, ceil] = [floor as u128, ceil as u128];

    if min == max {
        return None;
    }

    Some(
        nums.iter()
            .map(|num| {
                usize::try_from((ceil - floor) * (num - min) / (max - min) + floor)
                    .expect("Failed to convert u128 to usize")
            })
            .collect()
    )
}
