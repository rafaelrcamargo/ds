/// Handles cleaning and processing of ANSI escape sequences from Docker output
pub struct EscapeSequenceCleaner {
    partial_line: String
}

impl EscapeSequenceCleaner {
    pub fn new() -> Self {
        Self {
            partial_line: String::new()
        }
    }

    /// Processes a line and returns cleaned JSON if available
    pub fn process_line(&mut self, mut line: String) -> Option<String> {
        // Handle screen clearing sequences - these indicate we should print current stats
        if line.starts_with("\u{1b}[J\u{1b}[H") {
            line = line.replace("\u{1b}[J\u{1b}[H", "");
            if line.is_empty() {
                return None;
            }
        }

        // Handle cursor positioning sequences
        if line.starts_with("\u{1b}[H") {
            line = line.replace("\u{1b}[H", "");
            if line.is_empty() {
                return None;
            }
        }

        // Remove trailing escape sequences
        if line.ends_with("\u{1b}[K") {
            line = line.replace("\u{1b}[K", "").trim().to_string();
        }

        // Skip lines that are just whitespace or escape sequences
        if line.trim().is_empty() {
            return None;
        }

        // Handle partial JSON lines
        if !line.starts_with('{') && !self.partial_line.is_empty() {
            self.partial_line.push_str(&line);
            line = self.partial_line.clone();
            self.partial_line.clear();
        } else if line.starts_with('{') && !line.ends_with('}') {
            // This is the start of JSON but incomplete
            self.partial_line = line;
            return None;
        } else if !self.partial_line.is_empty() {
            // We have a partial line from before, complete it
            self.partial_line.push_str(&line);
            line = self.partial_line.clone();
            self.partial_line.clear();
        }

        // Skip lines that don't look like JSON
        if !line.starts_with('{') || !line.ends_with('}') {
            return None;
        }

        Some(line)
    }

    /// Check if the line indicates a screen clear event
    pub fn is_screen_clear_event(line: &str) -> bool { line.starts_with("\u{1b}[J\u{1b}[H") }
}
