mod cli;
mod data;
mod display;
mod error;
mod escape;
mod utils;

use data::DockerStats;
use display::StatsDisplay;
use error::{AppError, Result};
use escape::EscapeSequenceCleaner;
use utils::*;

use std::{
    io::{BufRead, BufReader},
    process::{Command, Stdio},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, TryRecvError},
        Arc
    },
    thread,
    time::{Duration, Instant}
};

static RUNNING: AtomicBool = AtomicBool::new(true);

fn main() {
    // Setup signal handling for graceful shutdown
    if let Err(e) = setup_signal_handling() {
        eprintln!("Warning: Failed to setup signal handling: {e}");
    }

    // Run the main application
    if let Err(e) = run_app() {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

fn setup_signal_handling() -> Result<()> {
    use signal_hook::{consts::SIGINT, iterator::Signals};

    let mut signals = Signals::new([SIGINT]).map_err(AppError::IoError)?;

    thread::spawn(move || {
        if signals.forever().next().is_some() {
            println!("\nReceived Ctrl+C, shutting down gracefully...");
            RUNNING.store(false, Ordering::SeqCst);
        }
    });

    Ok(())
}

fn run_app() -> Result<()> {
    let matches = cli::args().get_matches();
    let (compact, full) = (get_flag(&matches, "compact"), get_flag(&matches, "full"));
    let width = get_terminal_width();

    println!("Starting Docker stats monitor...");
    println!("Press Ctrl+C to exit");

    // Channel for communication between threads
    let (stats_sender, _stats_receiver) = mpsc::channel::<Vec<DockerStats>>();
    let (heartbeat_sender, heartbeat_receiver) = mpsc::channel::<()>();

    // Shared containers data
    let containers = Arc::new(std::sync::Mutex::new(Vec::<DockerStats>::new()));
    let display = Arc::new(StatsDisplay::new(width, compact, full));

    // Spawn display thread
    let display_containers = containers.clone();
    let display_handle = display.clone();
    let display_thread = thread::spawn(move || display_loop(heartbeat_receiver, display_containers, display_handle));

    // Spawn Docker stats reader thread
    let reader_containers = containers.clone();
    let reader_thread = thread::spawn(move || docker_stats_reader(matches, stats_sender, heartbeat_sender, reader_containers));

    // Wait for threads to complete
    let reader_result = reader_thread.join();
    let display_result = display_thread.join();

    // Handle thread results
    match (reader_result, display_result) {
        (Ok(Ok(())), Ok(())) => {
            println!("Application shut down successfully");
            Ok(())
        }
        (Ok(Err(e)), _) => Err(e),
        (Err(_), Ok(())) => Err(AppError::TerminalError("Reader thread panicked".to_string())),
        (Ok(Ok(())), Err(_)) => Err(AppError::TerminalError("Display thread panicked".to_string())),
        (Err(_), Err(_)) => Err(AppError::TerminalError("Both threads panicked".to_string()))
    }
}

fn docker_stats_reader(
    matches: clap::ArgMatches,
    _stats_sender: mpsc::Sender<Vec<DockerStats>>,
    heartbeat_sender: mpsc::Sender<()>,
    containers: Arc<std::sync::Mutex<Vec<DockerStats>>>
) -> Result<()> {
    let mut cmd = Command::new("docker")
        .args(build_command(matches))
        .stdout(Stdio::piped())
        .spawn()
        .map_err(AppError::from)?;

    let stdout = cmd
        .stdout
        .take()
        .ok_or_else(|| AppError::TerminalError("Failed to get stdout from docker command".to_string()))?;

    let reader = BufReader::new(stdout);
    let mut escape_cleaner = EscapeSequenceCleaner::new();

    for line_result in reader.lines() {
        if !RUNNING.load(Ordering::SeqCst) {
            break;
        }

        let line = line_result.map_err(AppError::from)?;

        // Send heartbeat
        let _ = heartbeat_sender.send(());

        // Check for screen clear event
        if EscapeSequenceCleaner::is_screen_clear_event(&line) {
            if let Ok(mut guard) = containers.lock() {
                guard.clear();
            }
        }

        // Process the line
        if let Some(clean_line) = escape_cleaner.process_line(line) {
            match serde_json::from_str::<DockerStats>(&clean_line) {
                Ok(stats) => {
                    if let Ok(mut containers_guard) = containers.lock() {
                        // Find existing container by name and update it, or add new one
                        if let Some(existing) = containers_guard.iter_mut().find(|c| c.name == stats.name) {
                            *existing = stats;
                        } else {
                            containers_guard.push(stats);
                        }
                    }
                }
                Err(e) => {
                    // Log parsing errors but don't stop the application
                    eprintln!("Warning: Failed to parse JSON: {e}");
                    continue;
                }
            }
        }
    }

    // Wait for docker command to finish
    let status = cmd.wait().map_err(AppError::from)?;

    // If we were interrupted (Ctrl+C), don't treat this as an error
    if !RUNNING.load(Ordering::SeqCst) {
        return Ok(());
    }

    if !status.success() {
        return Err(AppError::DockerNotRunning);
    }

    Ok(())
}

fn display_loop(heartbeat_receiver: Receiver<()>, containers: Arc<std::sync::Mutex<Vec<DockerStats>>>, display: Arc<StatsDisplay>) {
    let mut last_heartbeat = Instant::now();
    let timeout_duration = Duration::from_secs(3);

    loop {
        if !RUNNING.load(Ordering::SeqCst) {
            break;
        }

        // Check for heartbeat
        match heartbeat_receiver.try_recv() {
            Ok(()) => {
                last_heartbeat = Instant::now();
            }
            Err(TryRecvError::Empty) => {
                // No new heartbeat, check if we should timeout
                if last_heartbeat.elapsed() > timeout_duration {
                    if let Ok(mut guard) = containers.lock() {
                        guard.clear();
                    }
                    last_heartbeat = Instant::now();
                }
            }
            Err(TryRecvError::Disconnected) => {
                // Sender disconnected, exit
                break;
            }
        }

        // Display current stats
        if let Ok(guard) = containers.lock() {
            display.print_stats(&guard);
        }

        // Sleep briefly to avoid excessive CPU usage
        thread::sleep(Duration::from_millis(500));
    }
}
