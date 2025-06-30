use crate::calc_engine::{tokenize, Parser, EvaluationTrace};
use anyhow::Result;

pub fn evaluate_expression(expression: &str) -> Result<()> {
    let tokens = match tokenize(expression) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Error tokenizing expression: {}", e);
            return Ok(());
        }
    };

    let mut parser = Parser::new(tokens);
    let mut trace = EvaluationTrace::new(false); // No detailed trace for line mode

    match parser.parse(&mut trace) {
        Ok(result) => {
            println!("{}", result);
        }
        Err(e) => {
            eprintln!("Error evaluating expression: {}", e);
        }
    }
    Ok(())
}
