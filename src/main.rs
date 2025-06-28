// src/main.rs
mod calc_engine;
mod tui_mode;

use anyhow::Result;
use std::env;

fn print_help() {
    println!("Rust Calculator");
    println!("Usage: rustcalc [OPTION]");
    println!();
    println!("Options:");
    println!("  --tui, -t    Run in TUI mode (default)");
    println!("  --help, -h   Show this help");
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    match args.get(1).map(|s| s.as_str()) {
        Some("--tui") | Some("-t") | None => {
            tui_mode::run_tui()  // \u0412\u044b\u0437\u043e\u0432 \u043f\u0443\u0431\u043b\u0438\u0447\u043d\u043e\u0439 \u0444\u0443\u043d\u043a\u0446\u0438\u0438
        }
        Some("--help") | Some("-h") => {
            print_help();
            Ok(())
        }
        _ => {
            print_help();
            Ok(())
        }
    }
}
