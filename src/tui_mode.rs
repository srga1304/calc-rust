use crate::calc_engine::*;
use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, MouseEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    execute,
    cursor::{SetCursorStyle, Show},
};
use ratatui::{
    prelude::*,
    widgets::*,
    style::{Style, Color, Modifier},
    text::{Line, Span},
    Frame,
};
use std::{io, time::Duration};
use unicode_width::UnicodeWidthStr;

fn char_index_to_byte_index(s: &str, char_index: usize) -> usize {
    s.char_indices()
        .nth(char_index)
        .map(|(i, _)| i)
        .unwrap_or_else(|| s.len())
}

struct HistoryEntry {
    input: String,
    result: Result<f64, String>,
    detailed_steps: Vec<Step>,
    detailed_mode: bool,
    duration: std::time::Duration, // Добавлено поле для хранения времени вычисления
}

struct App {
    input: String,
    cursor_position: usize,
    history: Vec<HistoryEntry>,
    cursor_history: usize,
    should_quit: bool,
    show_help: bool,
    help_scroll: usize,
    list_height: usize,
    item_start_indices: Vec<usize>,
    history_scroll: usize,
    scroll_to_bottom: bool,
}

impl App {
    fn new() -> Self {
        App {
            input: String::new(),
            cursor_position: 0,
            history: Vec::new(),
            cursor_history: 0,
            should_quit: false,
            show_help: false,
            help_scroll: 0,
            list_height: 5,
            item_start_indices: Vec::new(),
            history_scroll: 0,
            scroll_to_bottom: false,
        }
    }

    fn submit(&mut self) {
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
                self.history_scroll = 0;
                return;
            }
            "help" => {
                self.show_help = true;
                self.input.clear();
                self.cursor_position = 0;
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
                duration: std::time::Duration::ZERO, // Нулевая длительность для ошибки
            });
            self.input.clear();
            self.cursor_position = 0;
            return;
        }

        // Замер времени начала вычисления
        let start_time = std::time::Instant::now();
        let mut trace = EvaluationTrace::new(detailed_mode);
        let result = match tokenize(processed_input) {
            Ok(tokens) => {
                let mut parser = Parser::new(tokens);
                parser.parse(&mut trace)
            }
            Err(e) => Err(e),
        };
        let duration = start_time.elapsed(); // Фиксация длительности вычисления

        self.history.push(HistoryEntry {
            input: processed_input.to_string(),
            result,
            detailed_steps: trace.steps,
            detailed_mode,
            duration, // Сохранение времени вычисления
        });

        self.cursor_history = self.history.len().saturating_sub(1);
        self.input.clear();
        self.cursor_position = 0;
        self.scroll_to_bottom = true;
    }

    fn move_cursor(&mut self, direction: i32) {
        match direction {
            -1 => self.cursor_position = self.cursor_position.saturating_sub(1),
            1 => self.cursor_position = (self.cursor_position + 1).min(self.input.chars().count()),
            _ => {}
        }
    }

    fn navigate_history(&mut self, direction: i32) {
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
        self.scroll_to_bottom = false;
    }

    fn scroll_history(&mut self, direction: i32) {
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
        self.scroll_to_bottom = false;
    }
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
    let mut stdout = io::stdout();
    enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen, SetCursorStyle::BlinkingBar)?;
    Terminal::new(CrosstermBackend::new(stdout)).map_err(Into::into)
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, Show, SetCursorStyle::DefaultUserShape)?;
    Ok(())
}

fn ui(frame: &mut Frame, app: &mut App) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![
            Constraint::Length(3),
            Constraint::Length(1),
            Constraint::Min(3),
        ])
        .split(frame.size());

    render_input(frame, app, layout[0]);
    render_status(frame, layout[1]);
    render_history(frame, app, layout[2]);
    app.list_height = layout[2].height as usize;
}

fn wrap_text(text: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current_line = String::new();
    let mut current_width = 0;

    for word in text.split_whitespace() {
        let word_width = word.width();

        if current_width + word_width + 1 > width && !current_line.is_empty() {
            lines.push(current_line.trim().to_string());
            current_line.clear();
            current_width = 0;
        }

        current_line.push_str(word);
        current_line.push(' ');
        current_width += word_width + 1;
    }

    if !current_line.is_empty() {
        lines.push(current_line.trim().to_string());
    }

    lines
}

fn render_history(frame: &mut Frame, app: &mut App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(" History ")
        .title_alignment(Alignment::Center);

    let inner_area = block.inner(area);
    frame.render_widget(block, area);

    if app.history.is_empty() {
        let empty_msg = Paragraph::new("No calculations yet. Enter an expression to see results here.")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);
        frame.render_widget(empty_msg, inner_area);
        return;
    }

    let mut items = Vec::new();
    app.item_start_indices.clear();

    let wrap_width = inner_area.width.saturating_sub(4) as usize;

    for (i, entry) in app.history.iter().enumerate() {
        app.item_start_indices.push(items.len());

        let is_selected = i == app.cursor_history;
        let input_style = Style::default()
            .fg(if is_selected { Color::Yellow } else { Color::Cyan });

        let input = format_with_spaces(&entry.input);
        let input_lines = wrap_text(&input, wrap_width);
        for (line_idx, line) in input_lines.iter().enumerate() {
            let mut result_spans = vec![];

            if line_idx == 0 {
                result_spans.push(Span::styled("➤ ", Style::default().fg(Color::Green)));
            } else {
                result_spans.push(Span::styled("  ", Style::default()));
            }

            result_spans.push(Span::styled(line.clone(), input_style));

            if line_idx == 0 {
                match &entry.result {
                    Ok(val) => {
                        let result_str = format!("{:.6}", val)
                            .trim_end_matches('0')
                            .trim_end_matches('.')
                            .to_string();
                        result_spans.push(Span::styled(" = ", Style::default().fg(Color::Gray)));
                        result_spans.push(Span::styled(
                            result_str,
                            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
                        ));
                    }
                    Err(e) => {
                        result_spans.push(Span::styled(" = ", Style::default().fg(Color::Gray)));
                        result_spans.push(Span::styled(
                            format!("Error: {}", e),
                            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
                        ));
                    }
                }
            }

            items.push(ListItem::new(Line::from(result_spans)));
        }

        if entry.detailed_mode {
            if !entry.detailed_steps.is_empty() {
                for (j, step) in entry.detailed_steps.iter().enumerate() {
                    let step_text = format!("     Step {}: {} = {:.6}", j + 1, step.operation, step.result)
                        .trim_end_matches('0')
                        .trim_end_matches('.')
                        .to_string();

                    let step_lines = wrap_text(&step_text, wrap_width);

                    for (step_idx, line) in step_lines.iter().enumerate() {
                        let prefix = if step_idx == 0 { "    └─ " } else { "       " };
                        let span = Span::styled(
                            format!("{}{}", prefix, line),
                            Style::default().fg(Color::DarkGray)
                        );
                        items.push(ListItem::new(Line::from(span)));
                    }
                }
            } else {
                match &entry.result {
                    Ok(_) => {}
                    Err(e) => {
                        let error_line = format!("    └─ Error: {}", e);
                        let error_lines = wrap_text(&error_line, wrap_width);
                        for (error_idx, line) in error_lines.iter().enumerate() {
                            let prefix = if error_idx == 0 { "    └─ " } else { "       " };
                            let span = Span::styled(
                                format!("{}{}", prefix, line),
                                Style::default().fg(Color::Red)
                            );
                            items.push(ListItem::new(Line::from(span)));
                        }
                    }
                }
            }

            // Добавляем время выполнения для детального режима
            let time_str = format!(
                "    └─ Time: {:.6} ms",
                entry.duration.as_secs_f64() * 1000.0
            );
            let time_lines = wrap_text(&time_str, wrap_width);
            for (time_idx, line) in time_lines.iter().enumerate() {
                let prefix = if time_idx == 0 { "    └─ " } else { "       " };
                let span = Span::styled(
                    format!("{}{}", prefix, line),
                    Style::default().fg(Color::Magenta) // Выделяем цветом
                );
                items.push(ListItem::new(Line::from(span)));
            }
        }

        if i < app.history.len() - 1 {
            let separator = Span::styled(
                "─".repeat(inner_area.width as usize),
                Style::default().fg(Color::DarkGray)
            );
            items.push(ListItem::new(Line::from(separator)));
        }
    }

    if app.scroll_to_bottom {
        app.history_scroll = items.len().saturating_sub(inner_area.height as usize);
        app.scroll_to_bottom = false;
    }

    let selected_index = if app.cursor_history < app.item_start_indices.len() {
        Some(app.item_start_indices[app.cursor_history])
    } else {
        None
    };

    let list = List::new(items)
        .block(Block::default())
        .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD));

    let mut state = ListState::default()
        .with_selected(selected_index)
        .with_offset(app.history_scroll);

    frame.render_stateful_widget(list, inner_area, &mut state);
}

fn render_status(frame: &mut Frame, area: Rect) {
    let keys = [
        ("Enter", "Calculate"),
        ("Navigate", "↑/↓ or PgUp/PgDn"),
        ("F1", "Help"),
        ("Esc", "Close Help"),
        ("Ctrl+C", "Quit"),
    ];

    let spans: Vec<Span> = keys
        .iter()
        .flat_map(|(key, desc)| {
            vec![
                Span::styled(
                    *key,
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!(" {} ", desc),
                    Style::default().fg(Color::DarkGray),
                ),
            ]
        })
        .collect();

    let line = Line::from(spans);
    let block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(Color::DarkGray));

    frame.render_widget(block, area);
    frame.render_widget(Paragraph::new(line), area);
}

fn render_input(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(" Expression ")
        .title_alignment(Alignment::Center);

    let inner_area = block.inner(area);
    frame.render_widget(block, area);

    let input_line = format!("> {}", app.input);
    let paragraph = Paragraph::new(input_line);

    frame.render_widget(paragraph, inner_area);

    let byte_position = char_index_to_byte_index(&app.input, app.cursor_position);
    let cursor_x = inner_area.x + 2 + byte_position as u16;
    let cursor_y = inner_area.y;
    frame.set_cursor(cursor_x, cursor_y);
}

fn render_help(frame: &mut Frame, app: &mut App) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(Color::Yellow))
        .title(" RustCalc Help ")
        .title_alignment(Alignment::Center)
        .style(Style::default().bg(Color::Black));

    let help_text = vec![
        Line::from(Span::styled("RustCalc - Advanced Terminal Calculator", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(Span::styled("Basic Operations:", Style::default().fg(Color::Cyan).add_modifier(Modifier::UNDERLINED))),
        Line::from("  + : Addition        (e.g., 5 + 3 = 8)"),
        Line::from("  - : Subtraction     (e.g., 10 - 4 = 6)"),
        Line::from("  * : Multiplication  (e.g., 6 * 7 = 42)"),
        Line::from("  / : Division        (e.g., 15 / 3 = 5)"),
        Line::from("  % : Modulo          (e.g., 10 % 3 = 1)"),
        Line::from("  ^ : Exponentiation  (e.g., 2 ^ 3 = 8)"),
        Line::from("  r : Root            (e.g., 8 r 3 = 2)"),
        Line::from(""),
        Line::from(Span::styled("Functions:", Style::default().fg(Color::Cyan).add_modifier(Modifier::UNDERLINED))),
        Line::from("  sin(x)   : Sine (x in degrees)"),
        Line::from("  cos(x)   : Cosine (x in degrees)"),
        Line::from("  tan(x)   : Tangent (x in degrees)"),
        Line::from("  asin(x)  : Arc sine (result in degrees)"),
        Line::from("  acos(x)  : Arc cosine (result in degrees)"),
        Line::from("  atan(x)  : Arc tangent (result in degrees)"),
        Line::from("  ln(x)    : Natural logarithm"),
        Line::from("  log(x)   : Base-10 logarithm"),
        Line::from("  exp(x)   : Exponential function"),
        Line::from("  abs(x)   : Absolute value"),
        Line::from("  sqrt(x)  : Square root"),
        Line::from("  floor(x) : Round down to nearest integer"),
        Line::from("  ceil(x)  : Round up to nearest integer"),
        Line::from("  round(x) : Round to nearest integer"),
        Line::from(""),
        Line::from(Span::styled("Hyperbolic Functions:", Style::default().fg(Color::Cyan).add_modifier(Modifier::UNDERLINED))),
        Line::from("  sinh(x)  : Hyperbolic sine"),
        Line::from("  cosh(x)  : Hyperbolic cosine"),
        Line::from("  tanh(x)  : Hyperbolic tangent"),
        Line::from("  asinh(x) : Inverse hyperbolic sine"),
        Line::from("  acosh(x) : Inverse hyperbolic cosine (x >= 1)"),
        Line::from("  atanh(x) : Inverse hyperbolic tangent (|x| < 1)"),
        Line::from(""),
        Line::from(Span::styled("Combinatorics:", Style::default().fg(Color::Cyan).add_modifier(Modifier::UNDERLINED))),
        Line::from("  fact(n)    : Factorial (n integer >=0)"),
        Line::from("  perm(n, k) : Permutations (n choose k)"),
        Line::from("  comb(n, k) : Combinations (n choose k)"),
        Line::from(""),
        Line::from(Span::styled("Statistical:", Style::default().fg(Color::Cyan).add_modifier(Modifier::UNDERLINED))),
        Line::from("  mean(a,b,...) : Arithmetic mean"),
        Line::from("  median(a,b,...) : Median"),
        Line::from("  stdev(a,b,...) : Standard deviation"),
        Line::from(""),
        Line::from(Span::styled("Constants:", Style::default().fg(Color::Cyan).add_modifier(Modifier::UNDERLINED))),
        Line::from("  pi : π (3.14159...)"),
        Line::from("  e  : Euler's number (2.71828...)"),
        Line::from(""),
        Line::from(Span::styled("Advanced Features:", Style::default().fg(Color::Cyan).add_modifier(Modifier::UNDERLINED))),
        Line::from("  details <expression> : Show step-by-step evaluation with time"),
        Line::from("  clear : Clear calculation history"),
        Line::from("  help : Show this help screen"),
        Line::from("  quit : Exit the calculator"),
        Line::from(""),
        Line::from(Span::styled("Navigation:", Style::default().fg(Color::Cyan).add_modifier(Modifier::UNDERLINED))),
        Line::from("  ← → : Move cursor left/right"),
        Line::from("  Home/End : Move to start/end of line"),
        Line::from("  ↑ ↓ : Navigate calculation history"),
        Line::from("  PgUp/PgDn : Page through history"),
        Line::from("  Mouse wheel : Scroll through history"),
        Line::from(""),
        Line::from(Span::styled("Examples:", Style::default().fg(Color::Cyan).add_modifier(Modifier::UNDERLINED))),
        Line::from("  sinh(1.5)"),
        Line::from("  fact(5)"),
        Line::from("  perm(10, 3)"),
        Line::from("  mean(1, 2, 3, 4, 5)"),
        Line::from("  details comb(8, 3)"),
        Line::from("  stdev(10, 12, 23, 23, 16)"),
    ];

    let paragraph = Paragraph::new(help_text)
        .block(block)
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true })
        .scroll((app.help_scroll as u16, 0));

    frame.render_widget(Clear, frame.size());
    frame.render_widget(paragraph, frame.size());
}

pub fn run_tui() -> Result<()> {
    let mut terminal = setup_terminal()?;
    let mut app = App::new();

    loop {
        terminal.draw(|f| {
            if app.show_help {
                render_help(f, &mut app);
            } else {
                ui(f, &mut app);
            }
        })?;

        if app.should_quit {
            break;
        }

        if event::poll(Duration::from_millis(50))? {
            match event::read()? {
                Event::Key(KeyEvent { code, kind, .. }) if kind == KeyEventKind::Press => {
                    if app.show_help {
                        match code {
                            KeyCode::Down => app.help_scroll = app.help_scroll.saturating_add(1),
                            KeyCode::Up => app.help_scroll = app.help_scroll.saturating_sub(1),
                            KeyCode::PageDown => app.help_scroll = app.help_scroll.saturating_add(10),
                            KeyCode::PageUp => app.help_scroll = app.help_scroll.saturating_sub(10),
                            KeyCode::Esc => {
                                app.show_help = false;
                                app.help_scroll = 0;
                            }
                            _ => {}
                        }
                    } else {
                        match code {
                            KeyCode::Char(c) => {
                                let byte_idx = char_index_to_byte_index(&app.input, app.cursor_position);
                                app.input.insert(byte_idx, c);
                                app.cursor_position += 1;
                            }
                            KeyCode::Backspace => {
                                if app.cursor_position > 0 {
                                    app.cursor_position -= 1;
                                    let byte_idx = char_index_to_byte_index(&app.input, app.cursor_position);
                                    let next_char = app.input[byte_idx..].chars().next();
                                    if let Some(c) = next_char {
                                        let end = byte_idx + c.len_utf8();
                                        app.input.drain(byte_idx..end);
                                    }
                                }
                            }
                            KeyCode::Delete => {
                                let byte_idx = char_index_to_byte_index(&app.input, app.cursor_position);
                                let next_char = app.input[byte_idx..].chars().next();
                                if let Some(c) = next_char {
                                    let end = byte_idx + c.len_utf8();
                                    app.input.drain(byte_idx..end);
                                }
                            }
                            KeyCode::Left => app.move_cursor(-1),
                            KeyCode::Right => app.move_cursor(1),
                            KeyCode::Home => app.cursor_position = 0,
                            KeyCode::End => app.cursor_position = app.input.chars().count(),
                            KeyCode::Up => app.navigate_history(-1),
                            KeyCode::Down => app.navigate_history(1),
                            KeyCode::PageUp => app.scroll_history(-1),
                            KeyCode::PageDown => app.scroll_history(1),
                            KeyCode::Enter => app.submit(),
                            KeyCode::F(1) => {
                                app.show_help = true;
                                app.help_scroll = 0;
                            }
                            KeyCode::Esc => app.show_help = false,
                            _ => {}
                        }
                    }
                }
                Event::Mouse(event) => {
                    if app.show_help {
                        match event.kind {
                            MouseEventKind::ScrollDown => app.help_scroll = app.help_scroll.saturating_add(3),
                            MouseEventKind::ScrollUp => app.help_scroll = app.help_scroll.saturating_sub(3),
                            _ => {}
                        }
                    } else {
                        match event.kind {
                            MouseEventKind::ScrollDown => {
                                app.history_scroll = app.history_scroll.saturating_add(3);
                            }
                            MouseEventKind::ScrollUp => {
                                app.history_scroll = app.history_scroll.saturating_sub(3);
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }
    }

    restore_terminal(&mut terminal)?;
    Ok(())
}
