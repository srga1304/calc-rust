use std::io;

fn main() {
    println!("Rust Console Calculator");
    println!("Available operations:");
    println!("  +   Addition");
    println!("  -   Subtraction");
    println!("  *   Multiplication");
    println!("  /   Division");
    println!("  ^   Power (a ^ b)");
    println!("  r   Root (nth root: r)");
    println!("  q   Quit");

    loop {
        println!("\nEnter operation (+, -, *, /, ^, r, q):");
        let mut op = String::new();
        io::stdin()
            .read_line(&mut op)
            .expect("Failed to read input");
        let op = op.trim();

        if op.eq_ignore_ascii_case("q") {
            println!("Goodbye!");
            break;
        }

        match op {
            "+" => addition(),
            "-" => subtraction(),
            "*" => multiplication(),
            "/" => division(),
            "^" => power(),
            "r" => root(),
            _ => println!("Unknown operation: {}", op),
        }
    }
}

fn read_number(prompt: &str) -> f64 {
    loop {
        println!("{}", prompt);
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read input");
        match input.trim().parse::<f64>() {
            Ok(num) => return num,
            Err(_) => println!("Invalid number, please try again."),
        }
    }
}

fn read_two_numbers() -> (f64, f64) {
    let a = read_number("Enter first number:");
    let b = read_number("Enter second number:");
    (a, b)
}

fn addition() {
    let (a, b) = read_two_numbers();
    let result = a + b;
    println!("Result: {} + {} = {}", a, b, result);
}

fn subtraction() {
    let (a, b) = read_two_numbers();
    let result = a - b;
    println!("Result: {} - {} = {}", a, b, result);
}

fn multiplication() {
    let (a, b) = read_two_numbers();
    let result = a * b;
    println!("Result: {} * {} = {}", a, b, result);
}

fn division() {
    let (a, b) = read_two_numbers();
    if b == 0.0 {
        println!("Error: Division by zero");
    } else {
        let result = a / b;
        println!("Result: {} / {} = {}", a, b, result);
    }
}

fn power() {
    let (a, b) = read_two_numbers();
    let result = a.powf(b);
    println!("Result: {} ^ {} = {}", a, b, result);
}

fn root() {
    let n = read_number("Enter root degree (n):");
    if n == 0.0 {
        println!("Error: Root degree cannot be zero");
        return;
    }
    let value = read_number("Enter radicand (value):");
    if value < 0.0 && n % 2.0 == 0.0 {
        println!("Error: Cannot take even root of negative number");
        return;
    }
    let result = value.powf(1.0 / n);
    println!("Result: {} root {} = {}", value, n, result);
}
