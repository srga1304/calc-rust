#[cfg(feature = "line")]
use crate::calc_engine::*;
#[cfg(feature = "line")]
use std::io::{Write, stdin, stdout};
#[cfg(feature = "line")]
use termion::{
    event::Key,
    input::TermRead,
    raw::IntoRawMode,
    cursor::{Goto, DetectCursorPos},
    clear::CurrentLine as ClearLine,
};

// Функция для преобразования позиции символа в байтовую позицию
#[cfg(feature = "line")]
fn char_index_to_byte_index(s: &str, char_index: usize) -> usize {
    s.char_indices()
        .nth(char_index)
        .map(|(i, _)| i)
        .unwrap_or_else(|| s.len())
}

#[cfg(feature = "line")]
pub fn run_line() {
    println!("Rust Console Calculator");
    println!("Supports: +, -, *, /, %, ^, r (root), functions (sin, cos, etc.)");
    println!("Constants: pi, e");
    println!("Navigation: ←/→, Backspace/Delete, Home/End, ↑/↓ for history");
    println!("Special commands: 'quit' to exit, 'clear' to reset history");
    println!("\rAdd 'details' before expression for step-by-step evaluation\n");

    let mut stdout = stdout().into_raw_mode().unwrap();
    let mut history: Vec<String> = Vec::new();
    let mut history_index = 0;

    loop {
        write!(stdout, "{}Expression: ", ClearLine).unwrap();
        stdout.flush().unwrap();

        let mut expression = String::new();
        let mut cursor_pos = 0;  // позиция курсора в символах
        let (_, initial_y) = stdout.cursor_pos().unwrap();

        let stdin = stdin();
        let mut keys = stdin.keys();

        loop {
            write!(
                stdout,
                "{}{}Expression: {}",
                Goto(1, initial_y),
                ClearLine,
                expression
            ).unwrap();

            // Вычисляем байтовую позицию для отображения курсора
            let byte_pos = char_index_to_byte_index(&expression, cursor_pos);
            write!(stdout, "{}", Goto((12 + byte_pos) as u16, initial_y)).unwrap();
            stdout.flush().unwrap();

            match keys.next().unwrap().unwrap() {
                Key::Char('\n') => break,
                Key::Char(c) => {
                    // Вставляем символ по правильной позиции
                    let byte_idx = char_index_to_byte_index(&expression, cursor_pos);
                    expression.insert(byte_idx, c);
                    cursor_pos += 1;
                }
                Key::Backspace if cursor_pos > 0 => {
                    cursor_pos -= 1;
                    let byte_idx = char_index_to_byte_index(&expression, cursor_pos);
                    let next_char = expression[byte_idx..].chars().next();
                    if let Some(c) = next_char {
                        let end = byte_idx + c.len_utf8();
                        expression.drain(byte_idx..end);
                    }
                }
                Key::Delete if cursor_pos < expression.chars().count() => {
                    let byte_idx = char_index_to_byte_index(&expression, cursor_pos);
                    let next_char = expression[byte_idx..].chars().next();
                    if let Some(c) = next_char {
                        let end = byte_idx + c.len_utf8();
                        expression.drain(byte_idx..end);
                    }
                }
                Key::Left if cursor_pos > 0 => cursor_pos -= 1,
                Key::Right if cursor_pos < expression.chars().count() => cursor_pos += 1,
                Key::Home => cursor_pos = 0,
                Key::End => cursor_pos = expression.chars().count(),
                Key::Up => {
                    if history_index > 0 {
                        history_index -= 1;
                        expression = history[history_index].clone();
                        cursor_pos = expression.chars().count();
                    }
                }
                Key::Down => {
                    if history_index < history.len().saturating_sub(1) {
                        history_index += 1;
                        expression = history[history_index].clone();
                        cursor_pos = expression.chars().count();
                    } else {
                        history_index = history.len();
                        expression.clear();
                        cursor_pos = 0;
                    }
                }
                _ => {}
            }
        }

        let input = expression.trim();
        if input.is_empty() {
            continue;
        }

        match input.to_lowercase().as_str() {
            "quit" | "exit" | "q" => {
                println!("\r\nGoodbye!");
                return;
            }
            "clear" | "reset" => {
                history.clear();
                history_index = 0;
                println!("\r\nHistory cleared\n");
                continue;
            }
            _ => {}
        }

        let (detailed_mode, processed_input) = if input.to_lowercase().starts_with("details ") {
            (true, input[8..].trim())
        } else if input.to_lowercase().ends_with(" details") {
            (true, input[..input.len()-7].trim())
        } else {
            (false, input)
        };

        if processed_input.is_empty() {
            println!("\r\nPlease enter a valid expression after 'details'");
            continue;
        }

        history.push(input.to_string());
        history_index = history.len();

        match tokenize(processed_input) {
            Ok(tokens) => {
                let mut parser = Parser::new(tokens);
                let mut trace = EvaluationTrace::new(detailed_mode);

                match parser.parse(&mut trace) {
                    Ok(result) => {
                        let formatted_expr = format_with_spaces(processed_input);
                        print!("\r\n  {} = {}\n", formatted_expr, result);

                        if detailed_mode && !trace.steps.is_empty() {
                            println!("\r\n  Step-by-step evaluation:");
                            for (i, step) in trace.steps.iter().enumerate() {
                                let formatted_op = format_with_spaces(&step.operation);
                                print!("\r  Step {}: {} = {}", i + 1, formatted_op, step.result);
                                stdout.flush().unwrap();
                                println!();
                            }
                            println!();
                        }
                    }
                    Err(e) => {
                        let formatted_expr = format_with_spaces(processed_input);
                        println!("\r\n  {} = Error: {}\n", formatted_expr, e);
                    }
                }
            }
            Err(e) => {
                let formatted_expr = format_with_spaces(processed_input);
                println!("\r\n  {} = Error: {}\n", formatted_expr, e);
            }
        }
    }
}
