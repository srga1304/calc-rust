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

fn main() {
    println!("Rust Console Calculator");
    println!("Supports: +, -, *, /, %, ^, r (root), functions (sin, cos, etc.)");
    println!("Constants: pi, e");
    println!("Navigation: ←/→, Backspace/Delete, Home/End, ↑/↓ for history");
    println!("Type 'quit' to exit or 'clear' to reset history\n");

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

        // Обработка команд после нажатия Enter
        match input.to_lowercase().as_str() {
            "quit" | "exit" | "q" => {
                println!("\nGoodbye!");
                return;
            }
            "clear" | "reset" => {
                history.clear();
                history_index = 0;
                println!("\nHistory cleared\n");
                continue;
            }
            _ => {}
        }

        history.push(input.to_string());
        history_index = history.len();

        match tokenize(input) {
            Ok(tokens) => {
                let mut parser = Parser::new(tokens);
                match parser.parse() {
                    Ok(result) => println!("\r\n  {} = {}\n", input, result),
                    Err(e) => println!("\r\n  {} = Error: {}\n", input, e),
                }
            }
            Err(e) => println!("\r\n  {} = Error: {}\n", input, e),
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

    fn parse(&mut self) -> Result<f64, String> {
        let result = self.expr()?;
        if self.current < self.tokens.len() {
            return Err("Unexpected tokens at end of expression".to_string());
        }
        Ok(result)
    }

    fn expr(&mut self) -> Result<f64, String> {
        let mut left = self.term()?;

        while self.current < self.tokens.len() {
            match self.tokens[self.current] {
                Token::Op('+') => {
                    self.current += 1;
                    left += self.term()?;
                }
                Token::Op('-') => {
                    self.current += 1;
                    left -= self.term()?;
                }
                _ => break,
            }
        }
        Ok(left)
    }

    fn term(&mut self) -> Result<f64, String> {
        let mut left = self.factor()?;

        while self.current < self.tokens.len() {
            match self.tokens[self.current] {
                Token::Op('*') => {
                    self.current += 1;
                    left *= self.factor()?;
                }
                Token::Op('/') => {
                    self.current += 1;
                    let right = self.factor()?;
                    if right == 0.0 {
                        return Err("Division by zero".to_string());
                    }
                    left /= right;
                }
                Token::Op('%') => {
                    self.current += 1;
                    left = (left as i64 % self.factor()? as i64) as f64;
                }
                _ => break,
            }
        }
        Ok(left)
    }

    fn factor(&mut self) -> Result<f64, String> {
        let base = self.power()?;

        if self.current < self.tokens.len() && self.tokens[self.current] == Token::Op('r') {
            self.current += 1;
            let exponent = self.power()?;
            if exponent == 0.0 {
                return Err("Root degree cannot be zero".to_string());
            }
            if base < 0.0 && exponent % 2.0 == 0.0 {
                return Err("Even root of negative number".to_string());
            }
            return Ok(base.powf(1.0 / exponent));
        }

        Ok(base)
    }

    fn power(&mut self) -> Result<f64, String> {
        let mut left = self.unary()?;

        if self.current < self.tokens.len() && self.tokens[self.current] == Token::Op('^') {
            self.current += 1;
            let right = self.power()?;
            left = left.powf(right);
        }

        Ok(left)
    }

    fn unary(&mut self) -> Result<f64, String> {
        let mut sign = 1.0;

        while self.current < self.tokens.len() {
            match self.tokens[self.current] {
                Token::Op('+') => self.current += 1,
                Token::Op('-') => {
                    sign = -sign;
                    self.current += 1;
                }
                _ => break,
            }
        }

        self.primary().map(|val| sign * val)
    }

    fn primary(&mut self) -> Result<f64, String> {
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
                let expr = self.expr()?;
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
                if name == "pi" { return Ok(PI); }
                if name == "e" { return Ok(E); }

                // Handle functions
                if self.current >= self.tokens.len() || self.tokens[self.current] != Token::LParen {
                    return Err(format!("Function '{}' requires parentheses", name));
                }
                self.current += 1;

                let arg = self.expr()?;

                if self.current >= self.tokens.len() || self.tokens[self.current] != Token::RParen {
                    return Err("Missing closing parenthesis for function".to_string());
                }
                self.current += 1;

                // Handle math functions with proper error checking
                match name.as_str() {
                    "sin" => Ok(arg.to_radians().sin()),
                    "cos" => Ok(arg.to_radians().cos()),
                    "tan" => Ok(arg.to_radians().tan()),
                    "asin" => {
                        if arg < -1.0 || arg > 1.0 {
                            Err("asin domain: [-1, 1]".to_string())
                        } else {
                            Ok(arg.asin().to_degrees())
                        }
                    }
                    "acos" => {
                        if arg < -1.0 || arg > 1.0 {
                            Err("acos domain: [-1, 1]".to_string())
                        } else {
                            Ok(arg.acos().to_degrees())
                        }
                    }
                    "atan" => Ok(arg.atan().to_degrees()),
                    "ln" => {
                        if arg <= 0.0 {
                            Err("ln domain: positive numbers".to_string())
                        } else {
                            Ok(arg.ln())
                        }
                    }
                    "log" => {
                        if arg <= 0.0 {
                            Err("log domain: positive numbers".to_string())
                        } else {
                            Ok(arg.log10())
                        }
                    }
                    "exp" => Ok(arg.exp()),
                    "abs" => Ok(arg.abs()),
                    "floor" => Ok(arg.floor()),
                    "ceil" => Ok(arg.ceil()),
                    "round" => Ok(arg.round()),
                    _ => Err(format!("Unknown function: '{}'", name)),
                }
            }
            _ => Err("Unexpected token".to_string()),
        }
    }
}
