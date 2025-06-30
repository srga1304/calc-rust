// src/main.rs
mod calc_engine;
mod tui_mode;
mod line_mode;
mod render_help; // Declare render_help as a module

use anyhow::Result;
use std::env;

fn print_help() {
    println!("Rust Calculator");
    println!("Usage: rustcalc [OPTION] [EXPRESSION]");
    println!();
    println!("Options:");
    println!("  --tui, -t    Run in TUI mode");
    println!("  --help, -h   Show this help");
    println!("\nIf no options are provided, or if an expression is given directly, it will be evaluated.");
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    match args.len() {
        1 => {
            // No arguments, show help
            print_help();
            Ok(())
        }
        _ => {
            let first_arg = args.get(1).map(|s| s.as_str());
            match first_arg {
                Some("--tui") | Some("-t") => {
                    tui_mode::run_tui()
                }
                Some("--help") | Some("-h") => {
                    print_help();
                    Ok(())
                }
                _ => {
                    // Treat remaining arguments as an expression
                    let expression = args[1..].join(" ");
                    line_mode::evaluate_expression(&expression)
                }
            }
        }
    }
}
