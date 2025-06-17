use std::f64::consts::{PI, E};

#[derive(Debug, PartialEq)]
pub enum Token {
    Number(f64),
    Op(char),
    Ident(String),
    LParen,
    RParen,
}

pub struct Step {
    pub operation: String,
    pub result: f64,
}

pub struct EvaluationTrace {
    pub steps: Vec<Step>,
    pub detailed_mode: bool,
}

impl EvaluationTrace {
    pub fn new(detailed_mode: bool) -> Self {
        EvaluationTrace {
            steps: Vec::new(),
            detailed_mode,
        }
    }

    pub fn add_step(&mut self, operation: String, result: f64) {
        if self.detailed_mode {
            self.steps.push(Step { operation, result });
        }
    }
}

pub fn format_with_spaces(expr: &str) -> String {
    let mut result = String::new();
    let mut last_char = '\0';
    let mut in_function = false;

    for c in expr.chars() {
        if "+-*/^%".contains(c) {
            if last_char != ' ' && last_char != '\0' {
                result.push(' ');
            }
            result.push(c);
            result.push(' ');
        }
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
        else if c.is_alphabetic() {
            if !in_function && last_char != ' ' && last_char != '\0' && !result.ends_with(' ') {
                result.push(' ');
            }
            result.push(c);
        }
        else {
            result.push(c);
        }

        last_char = c;
    }

    let parts: Vec<&str> = result.split_whitespace().collect();
    parts.join(" ")
}

pub fn tokenize(input: &str) -> Result<Vec<Token>, String> {
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

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, current: 0 }
    }

    pub fn parse(&mut self, trace: &mut EvaluationTrace) -> Result<f64, String> {
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
                    let operation = format!("{} + {}", left, right);
                    left += right;
                    trace.add_step(operation, left);
                }
                Token::Op('-') => {
                    self.current += 1;
                    let right = self.term(trace)?;
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
                    let operation = format!("{} / {}", left, right);
                    left /= right;
                    trace.add_step(operation, left);
                }
                Token::Op('%') => {
                    self.current += 1;
                    let right = self.factor(trace)?;
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

                if name == "pi" {
                    trace.add_step("pi".to_string(), PI);
                    return Ok(PI);
                }
                if name == "e" {
                    trace.add_step("e".to_string(), E);
                    return Ok(E);
                }

                if self.current >= self.tokens.len() || self.tokens[self.current] != Token::LParen {
                    return Err(format!("Function '{}' requires parentheses", name));
                }
                self.current += 1;

                let arg = self.expr(trace)?;

                if self.current >= self.tokens.len() || self.tokens[self.current] != Token::RParen {
                    return Err("Missing closing parenthesis for function".to_string());
                }
                self.current += 1;

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

                trace.add_step(format!("{}({})", name, arg), result);
                Ok(result)
            }
            _ => Err("Unexpected token".to_string()),
        }
    }
}
