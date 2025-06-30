use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Span;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

pub fn wrap_text(text: &str, width: usize) -> Vec<String> {
    if width == 0 {
        return vec!["".to_string()];
    }

    let mut lines = Vec::new();
    let mut current_line = String::new();
    let mut current_width = 0;

    for word in text.split_whitespace() {
        let word_width = word.width();

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
                    chunk_byte_len += c.len_utf8();
                }

                if !current_line.is_empty() {
                    lines.push(current_line.trim().to_string());
                    current_line.clear();
                    current_width = 0;
                }

                lines.push(chunk);
                remaining = &remaining[chunk_byte_len..];
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

pub fn format_number(x: f64) -> String {
    if x.abs() > 1e10 || (x.abs() < 1e-5 && x != 0.0) {
        format!("{:.6e}", x)
    } else {
        let s = format!("{:.6}", x);
        s.trim_end_matches('0')
            .trim_end_matches('.')
            .to_string()
    }
}

pub fn format_with_spaces(expr: &str) -> String {
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

pub fn is_math_function(word: &str) -> bool {
    matches!(
        word.to_lowercase().as_str(),
        "sin" | "cos" | "tan" | "asin" | "acos" | "atan" |
        "sinh" | "cosh" | "tanh" | "asinh" | "acosh" | "atanh" |
        "ln" | "log" | "exp" | "abs" | "sqrt" | "floor" | "ceil" | "round" |
        "fact" | "factorial" | "perm" | "npr" | "comb" | "ncr" | "mean" | "median" | "stdev" | "stddev" |
        "pi" | "e"
    )
}

pub fn highlight_functions(expr: &str, base_style: Style) -> Vec<Span<'static>> {
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
            if in_number {
                spans.push(Span::styled(current.clone(), number_style));
                current.clear();
                in_number = false;
            }

            current.push(c);
            in_function = true;
        } else if c.is_numeric() || c == '.' || c == 'e' || c == 'E' || (in_number && (c == '-' || c == '+')) {
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

            match c {
                '(' | ')' => {
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
