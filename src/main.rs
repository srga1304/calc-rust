use std::io::{Write, stdin, stdout};
use std::f64::consts::{PI, E};
use termion::{
    event::Key,
    input::TermRead,
    raw::IntoRawMode,
    cursor::{Goto, DetectCursorPos},
    clear::CurrentLine as ClearLine,
};

#[derive(Debug, PartialEq)]
enum Token {
    Number(f64),
    Op(char),
    Ident(String),
    LParen,
    RParen,
}

struct Step {
    operation: String,
    result: f64,
}

struct EvaluationTrace {
    steps: Vec<Step>,
    detailed_mode: bool,
}

impl EvaluationTrace {
    fn new(detailed_mode: bool) -> Self {
        EvaluationTrace {
            steps: Vec::new(),
            detailed_mode,
        }
    }

    fn add_step(&mut self, operation: String, result: f64) {
        if self.detailed_mode {
            self.steps.push(Step { operation, result });
        }
    }
}

// Функция для форматирования выражения с пробелами вокруг операторов
fn format_with_spaces(expr: &str) -> String {
    let mut result = String::new();
    let mut last_char = '\0';
    let mut in_function = false;

    for c in expr.chars() {
        // Проверяем, является ли символ оператором
        if "+-*/^%".contains(c) {
            // Добавляем пробелы вокруг оператора
            if last_char != ' ' && last_char != '\0' {
                result.push(' ');
            }
            result.push(c);
            result.push(' ');
        }
        // Обработка скобок и запятых
        else if c == '(' || c == ')' || c == ',' {
            if c == '(' {
                in_function = true;
            } else if c == ')' {
                in_function = false;
            }

            if last_char != ' ' {
                result.push(' ');
            }
            result.push(c);
            if c != ',' {
                result.push(' ');
            }
        }
        // Обработка букв (идентификаторов функций)
        else if c.is_alphabetic() {
            if !in_function && last_char != ' ' && last_char != '\0' && !result.ends_with(' ') {
                result.push(' ');
            }
            result.push(c);
        }
        // Обработка цифр и точек
        else {
            result.push(c);
        }

        last_char = c;
    }

    // Заменяем множественные пробелы на одинарные
    let parts: Vec<&str> = result.split_whitespace().collect();
    parts.join(" ")
}

fn main() {
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
        let mut cursor_pos = 0;
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
            write!(stdout, "{}", Goto((12 + cursor_pos) as u16, initial_y)).unwrap();
            stdout.flush().unwrap();

            match keys.next().unwrap().unwrap() {
                Key::Char('\n') => break,
                Key::Char(c) => {
                    expression.insert(cursor_pos, c);
                    cursor_pos += 1;
                }
                Key::Backspace if cursor_pos > 0 => {
                    cursor_pos -= 1;
                    expression.remove(cursor_pos);
                }
                Key::Delete if cursor_pos < expression.len() => {
                    expression.remove(cursor_pos);
                }
                Key::Left if cursor_pos > 0 => cursor_pos -= 1,
                Key::Right if cursor_pos < expression.len() => cursor_pos += 1,
                Key::Home => cursor_pos = 0,
                Key::End => cursor_pos = expression.len(),
                Key::Up => {
                    if history_index > 0 {
                        history_index -= 1;
                        expression = history[history_index].clone();
                        cursor_pos = expression.len();
                    }
                }
                Key::Down => {
                    if history_index < history.len().saturating_sub(1) {
                        history_index += 1;
                        expression = history[history_index].clone();
                        cursor_pos = expression.len();
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

        // Handle commands after Enter
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

        // Check for detailed mode request
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
                        // Форматируем выражение с пробелами
                        let formatted_expr = format_with_spaces(processed_input);

                        // Вывод основного результата
                        print!("\r\n  {} = {}\n", formatted_expr, result);

                        // Вывод пошаговых операций в детализированном режиме
                        if detailed_mode && !trace.steps.is_empty() {
                            println!("\r\n  Step-by-step evaluation:");
                            for (i, step) in trace.steps.iter().enumerate() {
                                // Форматируем каждую операцию
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

fn tokenize(input: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(&c) = chars.peek() {
        match c {
            ' ' | '\t' => {
                chars.next();
            }
            '(' => {
                tokens.push(Token::LParen);
                chars.next();
            }
            ')' => {
                tokens.push(Token::RParen);
                chars.next();
            }
            '+' | '-' | '*' | '/' | '^' | '%' | 'r' => {
                tokens.push(Token::Op(c));
                chars.next();
            }
            '0'..='9' | '.' => {
                let mut num_str = String::new();
                let mut has_dot = false;
                let mut has_exp = false;

                while let Some(&ch) = chars.peek() {
                    match ch {
                        '.' if has_dot => break,
                        '.' => {
                            has_dot = true;
                            num_str.push(ch);
                            chars.next();
                        }
                        'e' | 'E' if !has_exp => {
                            has_exp = true;
                            num_str.push(ch);
                            chars.next();

                            if let Some(&next_ch) = chars.peek() {
                                if next_ch == '+' || next_ch == '-' {
                                    num_str.push(next_ch);
                                    chars.next();
                                }
                            }
                        }
                        '0'..='9' => {
                            num_str.push(ch);
                            chars.next();
                        }
                        _ => break,
                    }
                }

                num_str.parse::<f64>()
                    .map(Token::Number)
                    .map_err(|_| format!("Invalid number: '{}'", num_str))
                    .and_then(|token| {
                        tokens.push(token);
                        Ok(())
                    })?;
            }
            'a'..='z' | 'A'..='Z' => {
                let mut ident = String::new();
                while let Some(&ch) = chars.peek() {
                    if ch.is_alphabetic() {
                        ident.push(ch);
                        chars.next();
                    } else {
                        break;
                    }
                }
                tokens.push(Token::Ident(ident));
            }
            _ => return Err(format!("Unknown character: '{}'", c)),
        }
    }
    Ok(tokens)
}

struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, current: 0 }
    }

    fn parse(&mut self, trace: &mut EvaluationTrace) -> Result<f64, String> {
        let result = self.expr(trace)?;
        if self.current < self.tokens.len() {
            return Err("Unexpected tokens at end of expression".to_string());
        }
        Ok(result)
    }

    fn expr(&mut self, trace: &mut EvaluationTrace) -> Result<f64, String> {
        let mut left = self.term(trace)?;

        while self.current < self.tokens.len() {
            match self.tokens[self.current] {
                Token::Op('+') => {
                    self.current += 1;
                    let right = self.term(trace)?;
                    // Добавляем пробелы вокруг оператора
                    let operation = format!("{} + {}", left, right);
                    left += right;
                    trace.add_step(operation, left);
                }
                Token::Op('-') => {
                    self.current += 1;
                    let right = self.term(trace)?;
                    // Добавляем пробелы вокруг оператора
                    let operation = format!("{} - {}", left, right);
                    left -= right;
                    trace.add_step(operation, left);
                }
                _ => break,
            }
        }
        Ok(left)
    }

    fn term(&mut self, trace: &mut EvaluationTrace) -> Result<f64, String> {
        let mut left = self.factor(trace)?;

        while self.current < self.tokens.len() {
            match self.tokens[self.current] {
                Token::Op('*') => {
                    self.current += 1;
                    let right = self.factor(trace)?;
                    // Добавляем пробелы вокруг оператора
                    let operation = format!("{} * {}", left, right);
                    left *= right;
                    trace.add_step(operation, left);
                }
                Token::Op('/') => {
                    self.current += 1;
                    let right = self.factor(trace)?;
                    if right == 0.0 {
                        return Err("Division by zero".to_string());
                    }
                    // Добавляем пробелы вокруг оператора
                    let operation = format!("{} / {}", left, right);
                    left /= right;
                    trace.add_step(operation, left);
                }
                Token::Op('%') => {
                    self.current += 1;
                    let right = self.factor(trace)?;
                    // Добавляем пробелы вокруг оператора
                    let operation = format!("{} % {}", left, right);
                    left = (left as i64 % right as i64) as f64;
                    trace.add_step(operation, left);
                }
                _ => break,
            }
        }
        Ok(left)
    }

    fn factor(&mut self, trace: &mut EvaluationTrace) -> Result<f64, String> {
        let base = self.power(trace)?;

        if self.current < self.tokens.len() && self.tokens[self.current] == Token::Op('r') {
            self.current += 1;
            let exponent = self.power(trace)?;
            if exponent == 0.0 {
                return Err("Root degree cannot be zero".to_string());
            }
            if base < 0.0 && exponent % 2.0 == 0.0 {
                return Err("Even root of negative number".to_string());
            }
            let result = base.powf(1.0 / exponent);
            // Добавляем пробелы вокруг оператора
            trace.add_step(format!("{} r {}", base, exponent), result);
            Ok(result)
        } else {
            Ok(base)
        }
    }

    fn power(&mut self, trace: &mut EvaluationTrace) -> Result<f64, String> {
        let left = self.unary(trace)?;

        if self.current < self.tokens.len() && self.tokens[self.current] == Token::Op('^') {
            self.current += 1;
            let right = self.power(trace)?;
            let result = left.powf(right);
            // Добавляем пробелы вокруг оператора
            trace.add_step(format!("{} ^ {}", left, right), result);
            Ok(result)
        } else {
            Ok(left)
        }
    }

    fn unary(&mut self, trace: &mut EvaluationTrace) -> Result<f64, String> {
        let mut sign = 1.0;
        let mut sign_changes = 0;

        while self.current < self.tokens.len() {
            match self.tokens[self.current] {
                Token::Op('+') => {
                    self.current += 1;
                }
                Token::Op('-') => {
                    sign = -sign;
                    sign_changes += 1;
                    self.current += 1;
                }
                _ => break,
            }
        }

        let mut result = self.primary(trace)?;
        result *= sign;

        if sign_changes > 0 {
            let sign_str = if sign == 1.0 { "+" } else { "-" };
            // Добавляем пробел между знаком и числом
            trace.add_step(format!("{} {}", sign_str, result.abs()), result);
        }

        Ok(result)
    }

    fn primary(&mut self, trace: &mut EvaluationTrace) -> Result<f64, String> {
        if self.current >= self.tokens.len() {
            return Err("Unexpected end of input".to_string());
        }

        match &self.tokens[self.current] {
            Token::Number(n) => {
                self.current += 1;
                Ok(*n)
            }
            Token::LParen => {
                self.current += 1;
                let expr = self.expr(trace)?;
                if self.current < self.tokens.len() && self.tokens[self.current] == Token::RParen {
                    self.current += 1;
                    Ok(expr)
                } else {
                    Err("Missing closing parenthesis".to_string())
                }
            }
            Token::Ident(ident) => {
                let name = ident.to_lowercase();
                self.current += 1;

                // Handle constants
                if name == "pi" {
                    trace.add_step("pi".to_string(), PI);
                    return Ok(PI);
                }
                if name == "e" {
                    trace.add_step("e".to_string(), E);
                    return Ok(E);
                }

                // Handle functions
                if self.current >= self.tokens.len() || self.tokens[self.current] != Token::LParen {
                    return Err(format!("Function '{}' requires parentheses", name));
                }
                self.current += 1;

                let arg = self.expr(trace)?;

                if self.current >= self.tokens.len() || self.tokens[self.current] != Token::RParen {
                    return Err("Missing closing parenthesis for function".to_string());
                }
                self.current += 1;

                // Handle math functions with proper error checking
                let result = match name.as_str() {
                    "sin" => arg.to_radians().sin(),
                    "cos" => arg.to_radians().cos(),
                    "tan" => arg.to_radians().tan(),
                    "asin" => {
                        if arg < -1.0 || arg > 1.0 {
                            return Err("asin domain: [-1, 1]".to_string());
                        }
                        arg.asin().to_degrees()
                    }
                    "acos" => {
                        if arg < -1.0 || arg > 1.0 {
                            return Err("acos domain: [-1, 1]".to_string());
                        }
                        arg.acos().to_degrees()
                    }
                    "atan" => arg.atan().to_degrees(),
                    "ln" => {
                        if arg <= 0.0 {
                            return Err("ln domain: positive numbers".to_string());
                        }
                        arg.ln()
                    }
                    "log" => {
                        if arg <= 0.0 {
                            return Err("log domain: positive numbers".to_string());
                        }
                        arg.log10()
                    }
                    "exp" => arg.exp(),
                    "abs" => arg.abs(),
                    "floor" => arg.floor(),
                    "ceil" => arg.ceil(),
                    "round" => arg.round(),
                    "sqrt" => {
                        if arg < 0.0 {
                            return Err("sqrt domain: non-negative numbers".to_string());
                        }
                        arg.sqrt()
                    }
                    _ => return Err(format!("Unknown function: '{}'", name)),
                };

                // Форматируем вызов функции с пробелами
                trace.add_step(format!("{}({})", name, arg), result);
                Ok(result)
            }
            _ => Err("Unexpected token".to_string()),
        }
    }
}
