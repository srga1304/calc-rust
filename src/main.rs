mod calc_engine;
mod tui_mode;
mod line_mode;

use anyhow::Result;
use std::env;

fn print_help() {
    println!("Rust Calculator");
    println!("Usage: rustcalc [OPTION]");
    println!();
    println!("Options:");
    println!("  --tui, -t    Run in TUI mode (default)");
    println!("  --line, -l   Run in line mode (console)");
    println!("  --help, -h   Show this help");
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    match args.get(1).map(|s| s.as_str()) {
        Some("--line") | Some("-l") => {
            #[cfg(feature = "line")]
            {
                line_mode::run_line();
                Ok(())
            }
            #[cfg(not(feature = "line"))]
            {
                eprintln!("Line mode is not supported in this build");
                Ok(())
            }
        }
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
