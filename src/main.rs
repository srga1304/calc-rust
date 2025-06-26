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
        Some("--tui") | Some("-t") => {
            #[cfg(feature = "tui")]
            tui_mode::run_tui()
        }
        Some("--help") | Some("-h") => {
            print_help();
            Ok(())
        }
        None => {
            #[cfg(feature = "tui")]
            tui_mode::run_tui()
        }
        _ => {
            print_help();
            Ok(())
        }
    }
}
