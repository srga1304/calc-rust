
use crate::tui_mode::*;

pub fn render_help(frame: &mut Frame, app: &mut App) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(Color::Yellow))
        .title(" RustCalc Help ")
        .title_alignment(Alignment::Center)
        .style(Style::default().bg(Color::Black));

    let help_text = vec![
        Line::from(Span::styled("RustCalc - Advanced Terminal Calculator", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(Span::styled("Basic Operations:", Style::default().fg(Color::Cyan).add_modifier(Modifier::UNDERLINED))),
        Line::from("  + : Addition        (e.g., 5 + 3 = 8)"),
        Line::from("  - : Subtraction     (e.g., 10 - 4 = 6)"),
        Line::from("  * : Multiplication  (e.g., 6 * 7 = 42)"),
        Line::from("  / : Division        (e.g., 15 / 3 = 5)"),
        Line::from("  % : Modulo          (e.g., 10 % 3 = 1)"),
        Line::from("  ^ : Exponentiation  (e.g., 2 ^ 3 = 8)"),
        Line::from("  r : Root            (e.g., 8 r 3 = 2)"),
        Line::from(""),
        Line::from(Span::styled("Functions:", Style::default().fg(Color::Cyan).add_modifier(Modifier::UNDERLINED))),
        Line::from("  sin(x)   : Sine (x in degrees)"),
        Line::from("  cos(x)   : Cosine (x in degrees)"),
        Line::from("  tan(x)   : Tangent (x in degrees)"),
        Line::from("  asin(x)  : Arc sine (result in degrees)"),
        Line::from("  acos(x)  : Arc cosine (result in degrees)"),
        Line::from("  atan(x)  : Arc tangent (result in degrees)"),
        Line::from("  ln(x)    : Natural logarithm"),
        Line::from("  log(x)   : Base-10 logarithm"),
        Line::from("  exp(x)   : Exponential function"),
        Line::from("  abs(x)   : Absolute value"),
        Line::from("  sqrt(x)  : Square root"),
        Line::from("  floor(x) : Round down to nearest integer"),
        Line::from("  ceil(x)  : Round up to nearest integer"),
        Line::from("  round(x) : Round to nearest integer"),
        Line::from(""),
        Line::from(Span::styled("Hyperbolic Functions:", Style::default().fg(Color::Cyan).add_modifier(Modifier::UNDERLINED))),
        Line::from("  sinh(x)  : Hyperbolic sine"),
        Line::from("  cosh(x)  : Hyperbolic cosine"),
        Line::from("  tanh(x)  : Hyperbolic tangent"),
        Line::from("  asinh(x) : Inverse hyperbolic sine"),
        Line::from("  acosh(x) : Inverse hyperbolic cosine (x >= 1)"),
        Line::from("  atanh(x) : Inverse hyperbolic tangent (|x| < 1)"),
        Line::from(""),
        Line::from(Span::styled("Combinatorics:", Style::default().fg(Color::Cyan).add_modifier(Modifier::UNDERLINED))),
        Line::from("  fact(n)    : Factorial (n integer >=0)"),
        Line::from("  perm(n, k) : Permutations (n choose k)"),
        Line::from("  comb(n, k) : Combinations (n choose k)"),
        Line::from(""),
        Line::from(Span::styled("Statistical:", Style::default().fg(Color::Cyan).add_modifier(Modifier::UNDERLINED))),
        Line::from("  mean(a,b,...) : Arithmetic mean"),
        Line::from("  median(a,b,...) : Median"),
        Line::from("  stdev(a,b,...) : Standard deviation"),
        Line::from(""),
        Line::from(Span::styled("Constants:", Style::default().fg(Color::Cyan).add_modifier(Modifier::UNDERLINED))),
        Line::from("  pi : π (3.14159...)"),
        Line::from("  e  : Euler's number (2.71828...)"),
        Line::from(""),
        Line::from(Span::styled("Advanced Features:", Style::default().fg(Color::Cyan).add_modifier(Modifier::UNDERLINED))),
        Line::from("  details <expression> : Show step-by-step evaluation with time"),
        Line::from("  clear : Clear calculation history"),
        Line::from("  Ctrl+U : Clear current input"),
        Line::from("  help : Show this help screen"),
        Line::from("  quit : Exit the calculator"),
        Line::from(""),
        Line::from(Span::styled("Navigation:", Style::default().fg(Color::Cyan).add_modifier(Modifier::UNDERLINED))),
        Line::from("  ← → : Move cursor left/right"),
        Line::from("  Ctrl+←/→ : Move cursor by words"),
        Line::from("  Home/End : Move to start/end of line"),
        Line::from("  ↑ ↓ : Navigate calculation history"),
        Line::from("  PgUp/PgDn : Page through history"),
        Line::from("  Mouse wheel : Scroll through history"),
        Line::from(""),
        Line::from(Span::styled("Examples:", Style::default().fg(Color::Cyan).add_modifier(Modifier::UNDERLINED))),
        Line::from("  sinh(1.5)"),
        Line::from("  fact(5)"),
        Line::from("  perm(10, 3)"),
        Line::from("  mean(1, 2, 3, 4, 5)"),
        Line::from("  details comb(8, 3)"),
        Line::from("  stdev(10, 12, 23, 23, 16)"),
    ];

    let paragraph = Paragraph::new(help_text)
        .block(block)
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true })
        .scroll((app.help_scroll as u16, 0));

    frame.render_widget(Clear, frame.size());
    frame.render_widget(paragraph, frame.size());
}
