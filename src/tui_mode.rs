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
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

// Минимальные размеры терминала
const MIN_TERMINAL_WIDTH: u16 = 50;
const MIN_TERMINAL_HEIGHT: u16 = 10;

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
    duration: std::time::Duration,
}

struct App {
    input: String,
    cursor_position: usize,
    input_scroll: usize,  // Горизонтальное смещение ввода
    history: Vec<HistoryEntry>,
    cursor_history: usize,
    should_quit: bool,
    show_help: bool,
    help_scroll: usize,
    list_height: usize,
    item_start_indices: Vec<usize>,
    history_scroll: usize,
    scroll_to_bottom: bool,
    terminal_too_small: bool,
}

impl App {
    fn new() -> Self {
        App {
            input: String::new(),
            cursor_position: 0,
            input_scroll: 0,  // Начальное смещение = 0
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
                self.input_scroll = 0;  // Сброс скролла
                return;
            }
            "help" => {
                self.show_help = true;
                self.input.clear();
                self.cursor_position = 0;
                self.input_scroll = 0;  // Сброс скролла
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
            self.input_scroll = 0;  // Сброс скролла
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
        self.input_scroll = 0;  // Сброс скролла
        self.scroll_to_bottom = true;
    }

    fn move_cursor(&mut self, direction: i32) {
        match direction {
            -1 => {
                if self.cursor_position > 0 {
                    self.cursor_position -= 1;
                    // Если курсор уходит за левую границу видимой области
                    if self.cursor_position < self.input_scroll {
                        self.input_scroll = self.cursor_position;
                    }
                }
            }
            1 => {
                if self.cursor_position < self.input.chars().count() {
                    self.cursor_position += 1;
                }
            }
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
        self.input_scroll = 0;  // Сброс скролла при навигации по истории
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
        self.input_scroll = 0;  // Сброс скролла
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
    let terminal_size = frame.size();

    // Проверяем размер терминала
    app.terminal_too_small = terminal_size.width < MIN_TERMINAL_WIDTH ||
                             terminal_size.height < MIN_TERMINAL_HEIGHT;

    if app.terminal_too_small {
        render_resize_message(frame, terminal_size);
        return;
    }

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![
            Constraint::Length(3),
            Constraint::Length(1),
            Constraint::Min(3),
        ])
        .split(terminal_size);

    render_input(frame, app, layout[0]);
    render_status(frame, layout[1]);
    render_history(frame, app, layout[2]);
    app.list_height = layout[2].height as usize;
}

fn render_resize_message(frame: &mut Frame, area: Rect) {
    let message = format!(
        "Terminal too small! Min size: {}x{}. Current: {}x{}",
        MIN_TERMINAL_WIDTH,
        MIN_TERMINAL_HEIGHT,
        area.width,
        area.height
    );

    let text = vec![
        Line::from(Span::styled(
            message,
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Please resize your terminal window",
            Style::default().fg(Color::Yellow)
        )),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red))
        .title(" Resize Required ")
        .title_alignment(Alignment::Center);

    let paragraph = Paragraph::new(text)
        .block(block)
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });

    frame.render_widget(paragraph, area);
}

fn wrap_text(text: &str, width: usize) -> Vec<String> {
    if width == 0 {
        return vec!["".to_string()];
    }

    let mut lines = Vec::new();
    let mut current_line = String::new();
    let mut current_width = 0;

    for word in text.split_whitespace() {
        let word_width = word.width();

        // Если слово слишком длинное, разбиваем его
        if word_width > width {
            let mut remaining = word;
            while !remaining.is_empty() {
                let mut chunk = String::new();
                let mut chunk_width = 0;
                let mut chunk_byte_len = 0;

                for c in remaining.chars() {
                    let char_width = UnicodeWidthChar::width_cjk(c).unwrap_or(1);
                    if chunk_width + char_width > width {
                        break;
                    }
                    chunk.push(c);
                    chunk_width += char_width;
                    chunk_byte_len += c.len_utf8(); // Сохраняем длину в байтах
                }

                if !current_line.is_empty() {
                    lines.push(current_line.trim().to_string());
                    current_line.clear();
                    current_width = 0;
                }

                lines.push(chunk);
                remaining = &remaining[chunk_byte_len..]; // Используем сохраненную длину
            }
            continue;
        }

        if current_width + word_width + 1 > width && !current_line.is_empty() {
            lines.push(current_line.trim().to_string());
            current_line.clear();
            current_width = 0;
        }

        if !current_line.is_empty() {
            current_line.push(' ');
            current_width += 1;
        }

        current_line.push_str(word);
        current_width += word_width;
    }

    if !current_line.is_empty() {
        lines.push(current_line.trim().to_string());
    }

    lines
}

fn format_number(x: f64) -> String {
    if x.abs() > 1e10 || (x.abs() < 1e-5 && x != 0.0) {
        format!("{:.6e}", x)
    } else {
        let s = format!("{:.6}", x);
        s.trim_end_matches('0')
            .trim_end_matches('.')
            .to_string()
    }
}

fn format_with_spaces(expr: &str) -> String {
    let mut result = String::new();
    let mut last_char = '\0';
    let mut in_function = false;

    for c in expr.chars() {
        match c {
            '+' | '-' | '*' | '/' | '^' | '%' | 'r' => {
                if last_char != ' ' && last_char != '\0' {
                    result.push(' ');
                }
                result.push(c);
                result.push(' ');
                last_char = ' ';
            }
            '(' => {
                if in_function {
                    result.push(c);
                } else {
                    if last_char != ' ' && last_char != '\0' {
                        result.push(' ');
                    }
                    result.push(c);
                }
                in_function = false;
                last_char = '(';
            }
            ')' | ',' => {
                result.push(c);
                if c == ',' {
                    result.push(' ');
                }
                last_char = c;
            }
            _ if c.is_whitespace() => {
                continue;
            }
            _ => {
                if c.is_alphabetic() {
                    in_function = true;
                } else if last_char == ')' || (c.is_numeric() && last_char.is_alphabetic()) {
                    result.push(' ');
                }
                result.push(c);
                last_char = c;
            }
        }
    }

    result.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn is_math_function(word: &str) -> bool {
    matches!(
        word.to_lowercase().as_str(),
        "sin" | "cos" | "tan" | "asin" | "acos" | "atan" |
        "sinh" | "cosh" | "tanh" | "asinh" | "acosh" | "atanh" |
        "ln" | "log" | "exp" | "abs" | "sqrt" | "floor" | "ceil" | "round" |
        "fact" | "perm" | "comb" | "mean" | "median" | "stdev" |
        "pi" | "e"
    )
}

fn highlight_functions(expr: &str, base_style: Style) -> Vec<Span<'static>> {
    let function_style = Style::default()
        .fg(Color::LightBlue)
        .add_modifier(Modifier::BOLD);

    let operator_style = Style::default()
        .fg(Color::Yellow)
        .add_modifier(Modifier::BOLD);

    let number_style = Style::default()
        .fg(Color::LightGreen);

    let mut spans = Vec::new();
    let mut current = String::new();
    let mut in_function = false;
    let mut in_number = false;

    for c in expr.chars() {
        if c.is_alphabetic() {
            // Завершаем число, если начались буквы
            if in_number {
                spans.push(Span::styled(current.clone(), number_style));
                current.clear();
                in_number = false;
            }

            current.push(c);
            in_function = true;
        } else if c.is_numeric() || c == '.' || c == 'e' || c == 'E' || (in_number && (c == '-' || c == '+')) {
            // Завершаем функцию, если началось число
            if in_function {
                if is_math_function(&current) {
                    spans.push(Span::styled(current.clone(), function_style));
                } else {
                    spans.push(Span::styled(current.clone(), base_style));
                }
                current.clear();
                in_function = false;
            }

            current.push(c);
            in_number = true;
        } else {
            // Завершаем число или функцию перед оператором/скобкой
            if in_function {
                if is_math_function(&current) {
                    spans.push(Span::styled(current.clone(), function_style));
                } else {
                    spans.push(Span::styled(current.clone(), base_style));
                }
                current.clear();
                in_function = false;
            } else if in_number {
                spans.push(Span::styled(current.clone(), number_style));
                current.clear();
                in_number = false;
            }

            // Обработка операторов и скобок
            match c {
                '(' | ')' => {
                    // Скобки функций окрашиваем в цвет функции, если это вызов функции
                    if in_function {
                        spans.push(Span::styled(c.to_string(), function_style));
                    } else {
                        spans.push(Span::styled(c.to_string(), base_style));
                    }
                }
                '+' | '-' | '*' | '/' | '^' | '%' | 'r' => {
                    spans.push(Span::styled(c.to_string(), operator_style));
                }
                ',' => {
                    spans.push(Span::styled(c.to_string(), base_style));
                }
                ' ' => {
                    spans.push(Span::raw(" "));
                }
                _ => {
                    spans.push(Span::styled(c.to_string(), base_style));
                }
            }
        }
    }

    // Обработка оставшихся частей
    if in_function {
        if is_math_function(&current) {
            spans.push(Span::styled(current, function_style));
        } else {
            spans.push(Span::styled(current, base_style));
        }
    } else if in_number {
        spans.push(Span::styled(current, number_style));
    }

    spans
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
        let base_style = Style::default()
            .fg(if is_selected { Color::Yellow } else { Color::Cyan });

        let input = format_with_spaces(&entry.input);
        let input_lines = wrap_text(&input, wrap_width);

        for (line_idx, line) in input_lines.into_iter().enumerate() {
            let mut result_spans = vec![];

            if line_idx == 0 {
                result_spans.push(Span::styled("➤ ", Style::default().fg(Color::Green)));
            } else {
                result_spans.push(Span::styled("  ", Style::default()));
            }

            let expr_spans = highlight_functions(&line, base_style);
            result_spans.extend(expr_spans);

            if line_idx == 0 {
                match &entry.result {
                    Ok(val) => {
                        let result_str = format_number(*val);
                        result_spans.push(Span::styled(" = ", Style::default().fg(Color::Gray)));
                        result_spans.push(Span::styled(
                            result_str,
                            Style::default().fg(Color::LightMagenta).add_modifier(Modifier::BOLD)
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
                    let step_result = format_number(step.result);
                    let step_text = format!("   Step {}: {} = {}", j + 1, step.operation, step_result);
                    let step_lines = wrap_text(&step_text, wrap_width);

                    for (step_idx, line) in step_lines.into_iter().enumerate() {
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
                        let error_line = format!("    ─ Error: {}", e);
                        let error_lines = wrap_text(&error_line, wrap_width);
                        for (error_idx, line) in error_lines.into_iter().enumerate() {
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

            let time_str = format!(
                "    ─ Time: {:.6} ms",
                entry.duration.as_secs_f64() * 1000.0
            );
            let time_lines = wrap_text(&time_str, wrap_width);
            for (time_idx, line) in time_lines.into_iter().enumerate() {
                let prefix = if time_idx == 0 { "    └─ " } else { "       " };
                let span = Span::styled(
                    format!("{}{}", prefix, line),
                    Style::default().fg(Color::Magenta)
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
        ("↑/↓ or PgUp/PgDn", "Navigate"),
        ("F1", "Help"),
        ("Esc", "Close Help"),
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

fn render_input(frame: &mut Frame, app: &mut App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(" Expression ")
        .title_alignment(Alignment::Center);

    let inner_area = block.inner(area);
    frame.render_widget(block, area);

    // Рассчитываем видимую область ввода
    let prefix = "> ";
    let input = &app.input;
    let cursor_char_pos = app.cursor_position;
    let visible_width = inner_area.width.saturating_sub(2) as usize; // 2 символа для "> "

    // Автоматическая корректировка скролла при перемещении курсора
    if cursor_char_pos < app.input_scroll {
        app.input_scroll = cursor_char_pos;
    } else if cursor_char_pos >= app.input_scroll + visible_width {
        app.input_scroll = cursor_char_pos - visible_width + 1;
    }

    // Формируем видимую часть строки
    let visible_input: String = input.chars()
        .skip(app.input_scroll)
        .take(visible_width)
        .collect();

    // Создаем отображаемую строку с префиксом
    let display_line = format!("{}{}", prefix, visible_input);

    // Рассчитываем позицию курсора в видимой области
    let visible_cursor_pos = cursor_char_pos - app.input_scroll;
    let visible_before_cursor: String = input.chars()
        .skip(app.input_scroll)
        .take(visible_cursor_pos)
        .collect();

    let cursor_x = prefix.width() + visible_before_cursor.width();

    // Отображаем ввод
    let paragraph = Paragraph::new(display_line);
    frame.render_widget(paragraph, inner_area);

    // Устанавливаем позицию курсора
    frame.set_cursor(
        inner_area.x + cursor_x as u16,
        inner_area.y
    );
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
                            KeyCode::Home => {
                                app.cursor_position = 0;
                                app.input_scroll = 0;  // Сброс скролла при переходе в начало
                            }
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
                            MouseEventKind::ScrollRight => {
                                // Горизонтальная прокрутка вправо
                                app.input_scroll = app.input_scroll.saturating_add(3);
                            }
                            MouseEventKind::ScrollLeft => {
                                // Горизонтальная прокрутка влево
                                app.input_scroll = app.input_scroll.saturating_sub(3);
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
