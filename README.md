# rustcalc

`rustcalc` is a command-line calculator written in Rust. It offers an interactive Text User Interface (TUI) for convenient calculations, as well as the ability to evaluate single expressions directly from the command line.

## Features

*   **Interactive TUI (Text User Interface):** Provides an interactive terminal interface for entering expressions and viewing results in real-time.
*   **Command-Line Evaluation:** Evaluate expressions directly from the command line.
*   **Supported Operations:**
    *   **Basic Arithmetic:**
        *   Addition (`+`)
        *   Subtraction (`-`)
        *   Multiplication (`*`)
        *   Division (`/`)
        *   Modulo (`%`)
        *   Parentheses for grouping expressions (`()`)
    *   **Exponents and Roots:**
        *   Power (`^`, e.g., `2^3` for 2 to the power of 3)
        *   N-th Root (`r`, e.g., `8 r 3` for the cube root of 8)
        *   Square Root (`sqrt(x)`)
    *   **Constants:**
        *   `pi` (mathematical constant π)
        *   `e` (Euler's number)
    *   **Trigonometric Functions:**
        *   `sin(x)` (sine, x in radians)
        *   `cos(x)` (cosine, x in radians)
        *   `tan(x)` (tangent, x in radians)
        *   `asin(x)` (arcsine, returns degrees)
        *   `acos(x)` (arccosine, returns degrees)
        *   `atan(x)` (arctangent, returns degrees)
    *   **Hyperbolic Functions:**
        *   `sinh(x)` (hyperbolic sine)
        *   `cosh(x)` (hyperbolic cosine)
        *   `tanh(x)` (hyperbolic tangent)
        *   `asinh(x)` (inverse hyperbolic sine)
        *   `acosh(x)` (inverse hyperbolic cosine)
        *   `atanh(x)` (inverse hyperbolic tangent)
    *   **Logarithmic and Exponential Functions:**
        *   `ln(x)` (natural logarithm)
        *   `log(x)` (base-10 logarithm)
        *   `exp(x)` (e to the power of x)
    *   **Basic Functions:**
        *   `abs(x)` (absolute value)
        *   `floor(x)` (rounds down to the nearest integer)
        *   `ceil(x)` (rounds up to the nearest integer)
        *   `round(x)` (rounds to the nearest integer)
    *   **Combinatorics:**
        *   `fact(n)` or `factorial(n)` (factorial)
        *   `perm(n, k)` or `npr(n, k)` (permutations)
        *   `comb(n, k)` or `ncr(n, k)` (combinations)
    *   **Statistical Functions:**
        *   `mean(a, b, ...)` (average of numbers)
        *   `median(a, b, ...)` (median of numbers)
        *   `stdev(a, b, ...)` or `stddev(a, b, ...)` (standard deviation)

## Project Structure

The `rustcalc` project is organized into modules for calculation logic (`calc_engine`), TUI components (`tui_mode`), and command-line evaluation (`line_mode`).

## Installation

To build and run `rustcalc`, you will need [Rust](https://www.rust-lang.org/tools/install) and its package manager Cargo installed on your system.

1.  **Get the source code:**
    ```bash
    git clone https://github.com/srga1304/calculator_project.git
    cd calculator_project # Assuming the cloned directory name is calculator_project
    ```
2.  **Build the project:**
    ```bash
    cargo build --release
    ```
    The executable will be located at `target/release/rustcalc`.

3.  **Global Installation (Optional):**
    To install `rustcalc` as a global command, allowing you to run it from any directory:
    ```bash
    cargo install --path .
    ```
    Ensure your Cargo bin directory (e.g., `~/.cargo/bin`) is in your system's PATH.

## Usage

`rustcalc` can be used in two primary ways:

### 1. Evaluate an Expression (Default)

To evaluate a single mathematical expression directly from the command line, simply provide the expression as an argument:

```bash
./target/release/rustcalc "2 + 2 * 3"
# Output: 8

./target/release/rustcalc "sin(pi/2) + cos(0)"
# Output: 2
```

### 2. TUI Mode

To enter the interactive Text User Interface (TUI) mode, use the `--tui` or `-t` flag:

```bash
./target/release/rustcalc --tui
# Or:
cargo run -- -t
```

In TUI mode, you can type mathematical expressions, and the results will be displayed on the screen.

### Help

To view the available command-line options, use the `--help` or `-h` flag:

```bash
./target/release/rustcalc --help
```

## Development

If you wish to contribute to or modify the project:

*   **Run in development mode:**
    ```bash
    cargo run
    ```
*   **Run tests:**
    ```bash
    cargo test
    ```

## License

This project is distributed under the [MIT License](LICENSE).