use clap::{Parser, Subcommand};
use colored::*;
use dice_roller::{roll, roll_with_seed};
use std::io::{self, Write};

#[derive(Parser)]
#[command(name = "dice_roller")]
#[command(about = "A comprehensive CLI dice roller supporting Roll20-style notation", long_about = None)]
#[command(version)]
struct Cli {
    /// Dice notation to roll (e.g., "1d20+5", "4d6kh3")
    #[arg(value_name = "DICE")]
    dice: Option<String>,

    /// Show verbose output with roll details
    #[arg(short, long)]
    verbose: bool,

    /// Show only the final total (quiet mode)
    #[arg(short, long)]
    quiet: bool,

    /// Repeat the roll N times
    #[arg(short = 'n', long, default_value = "1")]
    count: usize,

    /// Use a specific seed for deterministic rolls
    #[arg(short, long)]
    seed: Option<u64>,

    /// Disable colored output
    #[arg(long)]
    no_color: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Enter interactive mode
    Interactive,
}

fn main() {
    let cli = Cli::parse();

    // Disable colors if requested
    if cli.no_color {
        colored::control::set_override(false);
    }

    // Handle subcommands
    if let Some(Commands::Interactive) = cli.command {
        run_interactive(cli.verbose);
        return;
    }

    // Handle single roll
    if let Some(ref dice_notation) = cli.dice {
        run_roll(dice_notation, &cli);
    } else {
        // No dice notation provided, enter interactive mode
        run_interactive(cli.verbose);
    }
}

fn run_roll(notation: &str, cli: &Cli) {
    for i in 0..cli.count {
        if cli.count > 1 {
            println!("{} {}:", "Roll".cyan().bold(), i + 1);
        }

        let result = if let Some(seed) = cli.seed {
            roll_with_seed(notation, seed + i as u64)
        } else {
            roll(notation)
        };

        match result {
            Ok(roll_result) => {
                if cli.quiet {
                    println!("{}", roll_result.total);
                } else if cli.verbose {
                    print_verbose_result(&roll_result);
                } else {
                    print_result(&roll_result);
                }
            }
            Err(e) => {
                eprintln!("{} {}", "Error:".red().bold(), e);
                std::process::exit(1);
            }
        }

        if cli.count > 1 && i < cli.count - 1 {
            println!();
        }
    }
}

fn print_result(result: &dice_roller::RollResult) {
    println!(
        "{} {} = {}",
        "Roll:".cyan().bold(),
        result.expression,
        result.total.to_string().green().bold()
    );
}

fn print_verbose_result(result: &dice_roller::RollResult) {
    println!("{}", result.format(true));
}

fn run_interactive(verbose: bool) {
    println!("{}", "=== Dice Roller Interactive Mode ===".cyan().bold());
    println!("Enter dice notation to roll (e.g., '1d20+5', '4d6kh3')");
    println!("Type 'exit' or 'quit' to leave\n");

    loop {
        print!("{} ", ">".green().bold());
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            eprintln!("{}", "Error reading input".red());
            continue;
        }

        let input = input.trim();

        if input.is_empty() {
            continue;
        }

        if input == "exit" || input == "quit" {
            println!("Goodbye!");
            break;
        }

        if input == "help" {
            print_help();
            continue;
        }

        match roll(input) {
            Ok(result) => {
                if verbose {
                    print_verbose_result(&result);
                } else {
                    print_result(&result);
                }
            }
            Err(e) => {
                eprintln!("{} {}", "Error:".red().bold(), e);
            }
        }

        println!();
    }
}

fn print_help() {
    println!("{}", "Dice Notation Examples:".cyan().bold());
    println!("  {}  - Roll a 20-sided die plus 5", "1d20+5".yellow());
    println!("  {}  - Roll 4d6, keep highest 3", "4d6kh3".yellow());
    println!("  {}  - Roll 3d6, exploding on 6", "3d6!".yellow());
    println!("  {} - Roll 2d10, reroll 1s and 2s", "2d10r<2".yellow());
    println!("  {}  - Roll 10d6, count successes >= 5", "10d6>5".yellow());
    println!("  {}    - Roll 4 FATE dice", "4dF".yellow());
    println!("\n{}:", "Commands".cyan().bold());
    println!("  {}     - Show this help", "help".yellow());
    println!("  {}     - Exit interactive mode", "exit".yellow());
}
