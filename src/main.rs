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
        println!("\nENTER OPERATOR (+, -, *, /, ^, r, f!, log10, ln, exp)") }


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
            "+" | "-" | "*" | "/" | "^" => {
                // For these operations, read two numbers
                let (a, b) = read_two_numbers();
                let result = match op {
                    "+" => a + b,
                    "-" => a - b,
                    "*" => a * b,
                    "/" => {
                        if b == 0.0 {
                            println!("Error: Division by zero");
                            continue;
                        } else {
                            a / b
                        }
                    }
                    "^" => a.powf(b),
                    _ => unreachable!(),
                };
                println!("Result: {} {} {} = {}", a, op, b, result);
            }
            "r" => {
                // Root: a-th root of b -> b.powf(1/a)
                println!("Enter root degree (n):");
                let n = read_number();
                if n == 0.0 {
                    println!("Error: Root degree cannot be zero");
                    continue;
                }
                println!("Enter radicand (value):");
                let value = read_number();
                // For even root of negative number, error
                if value < 0.0 && n % 2.0 == 0.0 {
                    println!("Error: Cannot take even root of negative number");
                    continue;
                }
                let result = value.powf(1.0 / n);
                println!("Result: {} r{} = {}", value, n, result);
            }
            _ => println!("Unknown operation: {}", op),
        }
    }
}

fn read_number() -> f64 {
    loop {
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read input");
        match input.trim().parse::<f64>() {
            Ok(num) => return num,
            Err(_) => println!("Invalid number, please try again:"),
        }
    }
}

fn read_two_numbers() -> (f64, f64) {
    println!("Enter first number:");
    let a = read_number();
    println!("Enter second number:");
    let b = read_number();
    (a, b)
}
