#[path = "../src/utils.rs"]
mod utils;

#[path = "../src/cli.rs"]
mod cli;

#[cfg(test)]
mod command {
    use crate::{cli, utils};

    #[test]
    fn empty() {
        let matches = cli::args().get_matches_from(vec!["ds"]);
        let command = utils::build_command(matches);

        assert_eq!(command, vec!["stats", "--format", "json"]);
    }

    #[test]
    fn containers() {
        let matches = cli::args().get_matches_from(vec!["ds", "123", "456"]);
        let command = utils::build_command(matches);

        assert_eq!(command, vec!["stats", "--format", "json", "123", "456"]);
    }

    #[test]
    fn containers_with_flags() {
        let matches = cli::args().get_matches_from(vec!["ds", "123", "-c", "456", "-f"]);
        let command = utils::build_command(matches);

        assert_eq!(command, vec!["stats", "--format", "json", "123", "456"]);
    }
}

#[cfg(test)]
mod misc {
    use crate::utils;
    use colored::Colorize;

    #[test]
    fn usize_to_status() {
        // Should be green
        assert_eq!(utils::usize_to_status(0, 10), "".green());
        assert_eq!(utils::usize_to_status(4, 10), "████".green());

        // Should be yellow
        assert_eq!(utils::usize_to_status(5, 10), "█████".yellow());
        assert_eq!(utils::usize_to_status(7, 10), "███████".yellow());

        // Should be red
        assert_eq!(utils::usize_to_status(8, 10), "████████".red());
        assert_eq!(utils::usize_to_status(10, 10), "██████████".red());
    }

    #[test]
    fn scale_between() {
        // Pitfalls
        assert_eq!(utils::scale_between(vec![0, 0], 1, 10), None);

        // Common cases
        assert_eq!(utils::scale_between(vec![1, 2], 1, 10), Some(vec![1, 10]));
        assert_eq!(utils::scale_between(vec![1, 2, 3], 1, 10), Some(vec![1, 5, 10]));

        // Should be squished in the range
        assert_eq!(utils::scale_between(vec![1, 2], 1, 1), Some(vec![1, 1]));
        assert_eq!(utils::scale_between(vec![1, 3, 2], 1, 1), Some(vec![1, 1, 1]));
    }

    #[test]
    fn fill_on_even() {
        // Pitfalls
        assert_eq!(utils::fill_on_even("-", 0, 0), "");
        assert_eq!(utils::fill_on_even("-", 0, 5), "");

        // Common cases
        assert_eq!(utils::fill_on_even("-", 5, 0), "- - -");
        assert_eq!(utils::fill_on_even("-", 5, 1), "- - ");
    }

    #[test]
    fn perc_to() {
        // Pitfalls
        assert_eq!(utils::perc_to_float("10".to_string()), 0_f32);

        // Common cases
        assert_eq!(utils::perc_to_float("10%".to_string()), 10_f32);
        assert_eq!(utils::perc_to_float("9237%".to_string()), 9237_f32);
    }

    #[test]
    fn balanced_split() {
        assert_eq!(utils::balanced_split(0), vec![0, 0]);
        assert_eq!(utils::balanced_split(1), vec![0, 1]);
        assert_eq!(utils::balanced_split(2), vec![1, 1]);
        assert_eq!(utils::balanced_split(3), vec![1, 2]);
    }

    #[test]
    fn get_terminal_width() {
        assert!(utils::get_terminal_width() > 0);
    }
}
