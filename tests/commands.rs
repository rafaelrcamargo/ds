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

    #[test]
    fn check_flags() {
        let matches = cli::args().get_matches_from(vec!["ds", "123", "-c", "456", "-f"]);

        assert!(cli::has_flag(&matches, "compact"));
        assert!(cli::has_flag(&matches, "full"));
    }
}
