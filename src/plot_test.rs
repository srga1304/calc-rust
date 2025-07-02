#!/usr/bin/env rust-script
//! ```cargo
//! [dependencies]
//! termplot = "0.1.1"
//! meval = "0.2.0"
//! ```

use termplot::{Domain, Plot, Size};
use meval::Expr;
use std::io::{self, Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    loop {
        print!("Enter a function f(x) (or 'exit' to quit): ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input == "exit" {
            break;
        }

        let expr = input.parse::<Expr>()?;
        let func = expr.bind("x")?;

        let mut plot = Plot::default();
        plot.set_domain(Domain(-10.0..10.0))
            .set_codomain(Domain(-10.0..10.0))
            .set_size(Size::new(200, 100))
            .set_title(input)
            .set_x_label("x")
            .set_y_label("y")
            .add_plot(Box::new(termplot::plot::Graph::new(move |x| {
                func(x)
            })));

        println!("\n{plot}");
    }
    Ok(())
}
