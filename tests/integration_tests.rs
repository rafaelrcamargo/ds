use docker_stats::{data::DockerStats, display::StatsDisplay, error::AppError, escape::EscapeSequenceCleaner, utils};
use std::io::{Error as IoError, ErrorKind};

#[cfg(test)]
mod error_tests {
    use super::*;

    #[test]
    fn display_docker_not_found() {
        let error = AppError::DockerNotFound;
        assert_eq!(format!("{error}"), "Docker command not found. Please install Docker.");
    }

    #[test]
    fn display_docker_not_running() {
        let error = AppError::DockerNotRunning;
        assert_eq!(format!("{error}"), "Docker daemon is not running. Please start Docker.");
    }

    #[test]
    fn display_json_parse_error() {
        let error = AppError::JsonParseError("invalid syntax".to_string());
        assert_eq!(format!("{error}"), "Failed to parse Docker stats: invalid syntax");
    }

    #[test]
    fn display_terminal_error() {
        let error = AppError::TerminalError("terminal size unknown".to_string());
        assert_eq!(format!("{error}"), "Terminal error: terminal size unknown");
    }

    #[test]
    fn convert_io_error_not_found() {
        let io_error = IoError::new(ErrorKind::NotFound, "docker command not found");
        let app_error = AppError::from(io_error);

        match app_error {
            AppError::DockerNotFound => {}
            _ => panic!("Expected DockerNotFound error")
        }
    }

    #[test]
    fn convert_io_error_connection_refused() {
        let io_error = IoError::new(ErrorKind::ConnectionRefused, "connection refused");
        let app_error = AppError::from(io_error);

        match app_error {
            AppError::DockerNotRunning => {}
            _ => panic!("Expected DockerNotRunning error")
        }
    }

    #[test]
    fn convert_generic_io_error() {
        let io_error = IoError::new(ErrorKind::PermissionDenied, "permission denied");
        let app_error = AppError::from(io_error);

        match app_error {
            AppError::IoError(_) => {}
            _ => panic!("Expected IoError variant")
        }
    }
}

#[cfg(test)]
mod utils_tests {
    use super::*;
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
        assert_eq!(utils::perc_to_float("10"), 0_f32);

        // Common cases
        assert_eq!(utils::perc_to_float("10%"), 10_f32);
        assert_eq!(utils::perc_to_float("9237%"), 9237_f32);
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

#[cfg(test)]
mod display_tests {
    use super::*;

    fn create_test_stats() -> DockerStats {
        DockerStats {
            block_io: "1.2kB / 0B".to_string(),
            cpu_perc: "25.5%".to_string(),
            id: "abc123".to_string(),
            mem_perc: "50.0%".to_string(),
            mem_usage: "512MB / 1GB".to_string(),
            name: "test-container".to_string(),
            net_io: "10kB / 5kB".to_string()
        }
    }

    #[test]
    fn stats_display_creation() {
        let _display = StatsDisplay::new(80, false, false);
        // Basic test to ensure StatsDisplay can be created
        // This tests the public constructor
    }

    #[test]
    fn stats_display_with_different_modes() {
        let _display1 = StatsDisplay::new(80, true, false); // compact mode
        let _display2 = StatsDisplay::new(80, false, true); // full mode
        let _display3 = StatsDisplay::new(120, true, true); // both modes

        // Test that different configurations can be created
        // More detailed testing would require refactoring print_stats to return strings
    }

    #[test]
    fn stats_display_with_various_widths() {
        let _display1 = StatsDisplay::new(40, false, false); // narrow
        let _display2 = StatsDisplay::new(80, false, false); // standard
        let _display3 = StatsDisplay::new(200, false, false); // wide

        // Test that different terminal widths are handled
    }

    #[test]
    fn docker_stats_creation() {
        let stats = create_test_stats();
        assert_eq!(stats.name, "test-container");
        assert_eq!(stats.cpu_perc, "25.5%");
        assert_eq!(stats.mem_perc, "50.0%");
    }
}

#[cfg(test)]
mod data_tests {
    use super::*;

    #[test]
    fn deserialize_valid_docker_stats() {
        let json = r#"{
            "BlockIO": "1.2kB / 0B",
            "CPUPerc": "0.50%",
            "ID": "abc123",
            "MemPerc": "2.34%",
            "MemUsage": "123MB / 456MB",
            "Name": "test-container",
            "NetIO": "10kB / 5kB"
        }"#;

        let stats: DockerStats = serde_json::from_str(json).unwrap();
        assert_eq!(stats.cpu_perc, "0.50%");
        assert_eq!(stats.mem_perc, "2.34%");
        assert_eq!(stats.name, "test-container");
        assert_eq!(stats.block_io, "1.2kB / 0B");
        assert_eq!(stats.net_io, "10kB / 5kB");
    }

    #[test]
    fn deserialize_with_missing_fields() {
        let json = r#"{
            "BlockIO": "0B / 0B",
            "CPUPerc": "0.00%",
            "ID": "abc123",
            "MemPerc": "0.00%",
            "MemUsage": "0B / 0B",
            "Name": "minimal-container"
        }"#;

        // Should fail due to missing NetIO field
        assert!(serde_json::from_str::<DockerStats>(json).is_err());
    }

    #[test]
    fn deserialize_with_special_container_names() {
        let json = r#"{
            "BlockIO": "0B / 0B",
            "CPUPerc": "1.00%",
            "ID": "abc123",
            "MemPerc": "2.00%",
            "MemUsage": "100MB / 1GB",
            "Name": "my-app_v1.2.3-test",
            "NetIO": "1kB / 2kB"
        }"#;

        let stats: DockerStats = serde_json::from_str(json).unwrap();
        assert_eq!(stats.name, "my-app_v1.2.3-test");
    }

    #[test]
    fn deserialize_with_high_values() {
        let json = r#"{
            "BlockIO": "10GB / 5GB",
            "CPUPerc": "150.50%",
            "ID": "abc123",
            "MemPerc": "95.67%",
            "MemUsage": "15GB / 16GB",
            "Name": "heavy-container",
            "NetIO": "1TB / 500GB"
        }"#;

        let stats: DockerStats = serde_json::from_str(json).unwrap();
        assert_eq!(stats.cpu_perc, "150.50%");
        assert_eq!(stats.mem_perc, "95.67%");
    }
}

#[cfg(test)]
mod escape_tests {
    use super::*;

    #[test]
    fn process_clean_json_line() {
        let mut cleaner = EscapeSequenceCleaner::new();
        let result = cleaner.process_line(r#"{"Name":"test","CPUPerc":"1.0%"}"#.to_string());
        assert_eq!(result, Some(r#"{"Name":"test","CPUPerc":"1.0%"}"#.to_string()));
    }

    #[test]
    fn process_line_with_trailing_escape() {
        let mut cleaner = EscapeSequenceCleaner::new();
        let result = cleaner.process_line("{\"Name\":\"test\",\"CPUPerc\":\"1.0%\"}\u{1b}[K".to_string());
        assert_eq!(result, Some(r#"{"Name":"test","CPUPerc":"1.0%"}"#.to_string()));
    }

    #[test]
    fn detect_screen_clear_events() {
        assert!(EscapeSequenceCleaner::is_screen_clear_event("\u{1b}[J\u{1b}[H"));
        assert!(EscapeSequenceCleaner::is_screen_clear_event("\u{1b}[J\u{1b}[H additional text"));
        assert!(!EscapeSequenceCleaner::is_screen_clear_event("normal text"));
        assert!(!EscapeSequenceCleaner::is_screen_clear_event("\u{1b}[K"));
        assert!(!EscapeSequenceCleaner::is_screen_clear_event("\u{1b}[H"));
    }
}
