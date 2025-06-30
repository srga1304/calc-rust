use super::app::App;
use anyhow::Result;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseEventKind};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame, Terminal,
};
use std::time::Duration;
use unicode_width::UnicodeWidthStr;
use super::helpers::{format_number, format_with_spaces, highlight_functions, wrap_text};
use crate::render_help::render_help; // Import the centralized render_help function

const MIN_TERMINAL_WIDTH: u16 = 50;
const MIN_TERMINAL_HEIGHT: u16 = 10;

pub fn run_ui_loop(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    app: &mut App,
) -> Result<()> {
    loop {
        terminal.draw(|f| {
            if app.show_help {
                render_help(f, app);
            } else {
                ui(f, app);
            }
        })?;

        if app.should_quit {
            break;
        }

        if crossterm::event::poll(Duration::from_millis(50))? {
            match crossterm::event::read()? {
                Event::Key(KeyEvent { code, modifiers, kind, .. }) if kind == KeyEventKind::Press => {
                    handle_key_event(app, code, modifiers);
                }
                Event::Mouse(event) => {
                    handle_mouse_event(app, event);
                }
                _ => {}
            }
        }
    }
    Ok(())
}

fn handle_key_event(app: &mut App, code: KeyCode, modifiers: KeyModifiers) {
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
            KeyCode::Char(c) if modifiers.is_empty() => {
                let byte_idx = App::char_index_to_byte_index(&app.input, app.cursor_position);
                app.input.insert(byte_idx, c);
                app.cursor_position += 1;
            }
            KeyCode::Backspace => {
                if app.cursor_position > 0 {
                    app.cursor_position -= 1;
                    let byte_idx = App::char_index_to_byte_index(&app.input, app.cursor_position);
                    let next_char = app.input[byte_idx..].chars().next();
                    if let Some(c) = next_char {
                        let end = byte_idx + c.len_utf8();
                        app.input.drain(byte_idx..end);
                    }
                }
            }
            KeyCode::Delete => {
                let byte_idx = App::char_index_to_byte_index(&app.input, app.cursor_position);
                let next_char = app.input[byte_idx..].chars().next();
                if let Some(c) = next_char {
                    let end = byte_idx + c.len_utf8();
                    app.input.drain(byte_idx..end);
                }
            }
            KeyCode::Left if modifiers.contains(KeyModifiers::CONTROL) => {
                app.move_cursor_by_words(-1);
            }
            KeyCode::Right if modifiers.contains(KeyModifiers::CONTROL) => {
                app.move_cursor_by_words(1);
            }
            KeyCode::Left => app.move_cursor(-1),
            KeyCode::Right => app.move_cursor(1),
            KeyCode::Home => {
                app.cursor_position = 0;
                app.input_scroll = 0;
            }
            KeyCode::End => {
                app.cursor_position = app.input.chars().count();
            }
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
            KeyCode::Char('u') | KeyCode::Char('U') if modifiers.contains(KeyModifiers::CONTROL) => {
                app.clear_input();
            }
            _ => {}
        }
    }
}

fn handle_mouse_event(app: &mut App, event: crossterm::event::MouseEvent) {
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

fn ui(frame: &mut Frame, app: &mut App) {
    let terminal_size = frame.size();

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
                result_spans.push(Span::styled("> ", Style::default().fg(Color::Green)));
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
                        let prefix = if step_idx == 0 { "    - " } else { "      " };
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
                        let error_line = format!("    - Error: {}", e);
                        let error_lines = wrap_text(&error_line, wrap_width);
                        for (error_idx, line) in error_lines.into_iter().enumerate() {
                            let prefix = if error_idx == 0 { "    - " } else { "      " };
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
                "    - Time: {:.6} ms",
                entry.duration.as_secs_f64() * 1000.0
            );
            let time_lines = wrap_text(&time_str, wrap_width);
            for (time_idx, line) in time_lines.into_iter().enumerate() {
                let prefix = if time_idx == 0 { "    - " } else { "      " };
                let span = Span::styled(
                    format!("{}{}", prefix, line),
                    Style::default().fg(Color::Magenta)
                );
                items.push(ListItem::new(Line::from(span)));
            }
        }

        if i < app.history.len() - 1 {
            let separator = Span::styled(
                "-".repeat(inner_area.width as usize),
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
        ("Up/Down or PgUp/PgDn", "Navigate"),
        ("F1", "Help"),
        ("Esc", "Close Help"),
        ("Ctrl+U", "Clear Input"),
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

    let visible_width = (inner_area.width.saturating_sub(2)) as usize;
    let total_chars = app.input.chars().count();
    app.adjust_input_scroll(visible_width);

    let visible_input: String = app.input
        .chars()
        .skip(app.input_scroll)
        .take(visible_width)
        .collect();

    let input_line = format!("> {}", visible_input);
    let paragraph = Paragraph::new(input_line);
    frame.render_widget(paragraph, inner_area);

    let visible_cursor = app.cursor_position.saturating_sub(app.input_scroll);
    let visible_prefix = visible_input.chars().take(visible_cursor).collect::<String>();
    let cursor_x = inner_area.x + 2 + visible_prefix.width() as u16;
    let cursor_y = inner_area.y;
    frame.set_cursor(cursor_x, cursor_y);

    let scroll_indicator_style = Style::default().fg(Color::DarkGray);

    if app.input_scroll > 0 {
        let left_indicator = Paragraph::new("<").style(scroll_indicator_style);
        frame.render_widget(left_indicator, Rect::new(inner_area.x, inner_area.y, 1, 1));
    }

    if total_chars > app.input_scroll + visible_width {
        let right_indicator = Paragraph::new(">").style(scroll_indicator_style);
        frame.render_widget(
            right_indicator,
            Rect::new(inner_area.x + inner_area.width - 1, inner_area.y, 1, 1),
        );
    }
}


