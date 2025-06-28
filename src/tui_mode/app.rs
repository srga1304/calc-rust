use crate::calc_engine::*;
use anyhow::Result;

pub struct HistoryEntry {
    pub input: String,
    pub result: Result<f64, String>,
    pub detailed_steps: Vec<Step>,
    pub detailed_mode: bool,
    pub duration: std::time::Duration,
}

pub struct App {
    pub input: String,
    pub cursor_position: usize,
    pub input_scroll: usize,
    pub history: Vec<HistoryEntry>,
    pub cursor_history: usize,
    pub should_quit: bool,
    pub show_help: bool,
    pub help_scroll: usize,
    pub list_height: usize,
    pub item_start_indices: Vec<usize>,
    pub history_scroll: usize,
    pub scroll_to_bottom: bool,
    pub terminal_too_small: bool,
}

impl App {
    pub fn new() -> Self {
        App {
            input: String::new(),
            cursor_position: 0,
            input_scroll: 0,
            history: Vec::new(),
            cursor_history: 0,
            should_quit: false,
            show_help: false,
            help_scroll: 0,
            list_height: 5,
            item_start_indices: Vec::new(),
            history_scroll: 0,
            scroll_to_bottom: false,
            terminal_too_small: false,
        }
    }

    pub fn adjust_input_scroll(&mut self, visible_width: usize) {
        let total_chars = self.input.chars().count();
        let cursor_pos = self.cursor_position;

        if cursor_pos < self.input_scroll {
            self.input_scroll = cursor_pos;
        }
        else if cursor_pos >= self.input_scroll + visible_width {
            self.input_scroll = cursor_pos - visible_width + 1;
        }

        if self.input_scroll > total_chars.saturating_sub(visible_width) {
            self.input_scroll = total_chars.saturating_sub(visible_width);
        }
    }

    pub fn submit(&mut self) {
        let input = self.input.trim();
        if input.is_empty() {
            return;
        }

        match input.to_lowercase().as_str() {
            "quit" | "exit" | "q" => {
                self.should_quit = true;
                return;
            }
            "clear" | "reset" => {
                self.history.clear();
                self.cursor_history = 0;
                self.input.clear();
                self.cursor_position = 0;
                self.input_scroll = 0;
                self.history_scroll = 0;
                return;
            }
            "help" => {
                self.show_help = true;
                self.input.clear();
                self.cursor_position = 0;
                self.input_scroll = 0;
                return;
            }
            _ => {}
        }

        let (detailed_mode, processed_input) = if input.to_lowercase().starts_with("details ") {
            (true, input[8..].trim())
        } else if input.to_lowercase().ends_with(" details") {
            (true, input[..input.len() - 7].trim())
        } else {
            (false, input)
        };

        if processed_input.is_empty() {
            self.history.push(HistoryEntry {
                input: input.to_string(),
                result: Err("Please enter a valid expression after 'details'".to_string()),
                detailed_steps: Vec::new(),
                detailed_mode: false,
                duration: std::time::Duration::ZERO,
            });
            self.input.clear();
            self.cursor_position = 0;
            self.input_scroll = 0;
            return;
        }

        let start_time = std::time::Instant::now();
        let mut trace = EvaluationTrace::new(detailed_mode);
        let result = match tokenize(processed_input) {
            Ok(tokens) => {
                let mut parser = Parser::new(tokens);
                parser.parse(&mut trace)
            }
            Err(e) => Err(e),
        };
        let duration = start_time.elapsed();

        self.history.push(HistoryEntry {
            input: processed_input.to_string(),
            result,
            detailed_steps: trace.steps,
            detailed_mode,
            duration,
        });

        self.cursor_history = self.history.len().saturating_sub(1);
        self.input.clear();
        self.cursor_position = 0;
        self.input_scroll = 0;
        self.scroll_to_bottom = true;
    }

    pub fn move_cursor(&mut self, direction: i32) {
        match direction {
            -1 => self.cursor_position = self.cursor_position.saturating_sub(1),
            1 => self.cursor_position = (self.cursor_position + 1).min(self.input.chars().count()),
            _ => {}
        }
    }

    pub fn move_cursor_by_words(&mut self, direction: i32) {
        if direction < 0 {
            let input_chars: Vec<char> = self.input.chars().collect();
            let mut pos = self.cursor_position;

            while pos > 0 && input_chars[pos - 1].is_whitespace() {
                pos -= 1;
            }

            while pos > 0 && !input_chars[pos - 1].is_whitespace() {
                pos -= 1;
            }

            self.cursor_position = pos;
        } else {
            let input_chars: Vec<char> = self.input.chars().collect();
            let mut pos = self.cursor_position;
            let len = input_chars.len();

            while pos < len && !input_chars[pos].is_whitespace() {
                pos += 1;
            }

            while pos < len && input_chars[pos].is_whitespace() {
                pos += 1;
            }

            self.cursor_position = pos.min(len);
        }
    }

    pub fn navigate_history(&mut self, direction: i32) {
        if direction < 0 && self.cursor_history > 0 {
            self.cursor_history -= 1;
        } else if direction > 0 && self.cursor_history < self.history.len().saturating_sub(1) {
            self.cursor_history += 1;
        }

        if self.cursor_history < self.history.len() {
            self.input = self.history[self.cursor_history].input.clone();
        } else {
            self.input.clear();
        }
        self.cursor_position = self.input.chars().count();
        self.input_scroll = 0;
        self.scroll_to_bottom = false;
    }

    pub fn scroll_history(&mut self, direction: i32) {
        let step = self.list_height.saturating_sub(1);
        if direction < 0 {
            self.cursor_history = self.cursor_history.saturating_sub(step);
        } else {
            self.cursor_history = self.cursor_history.saturating_add(step)
                .min(self.history.len().saturating_sub(1));
        }

        if self.cursor_history < self.history.len() {
            self.input = self.history[self.cursor_history].input.clone();
        }
        self.cursor_position = self.input.chars().count();
        self.input_scroll = 0;
        self.scroll_to_bottom = false;
    }

    pub fn clear_input(&mut self) {
        self.input.clear();
        self.cursor_position = 0;
        self.input_scroll = 0;
    }

    pub fn char_index_to_byte_index(s: &str, char_index: usize) -> usize {
        s.char_indices()
            .nth(char_index)
            .map(|(i, _)| i)
            .unwrap_or_else(|| s.len())
    }
}
