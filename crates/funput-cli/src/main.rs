//! `funput` — terminal dev tool driving `funput-engine` (Telex/VNI).
//!
//! Not a real IME: no keyboard hooks, no injecting into other apps. It feeds an
//! input string through the engine and prints what a platform shell would show,
//! for quick checks, debugging, and CI.

mod cli;
mod coverage;
mod encode;
mod render;
mod repl;
mod sim;

use std::path::PathBuf;

use clap::Parser;

use cli::{Cli, Command};
use render::steps_table;

fn main() {
    match Cli::parse().command {
        Command::Run { input, opts } => {
            let simulation = sim::simulate(opts.method.into(), &input);
            if opts.steps {
                println!("{}", steps_table(&simulation));
            } else {
                println!("{}", simulation.app_text);
            }
        }
        Command::Repl { opts } => repl::run(opts.method.into(), opts.steps),
        Command::Coverage {
            corpus,
            json,
            show_mismatches,
            limit,
        } => {
            let path = corpus.unwrap_or_else(|| PathBuf::from("benchmarks/sample.txt"));
            if let Err(e) = coverage::run(&path, json, show_mismatches, limit) {
                eprintln!("coverage: cannot read corpus {}: {e}", path.display());
                std::process::exit(1);
            }
        }
    }
}
