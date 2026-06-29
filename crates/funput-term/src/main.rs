//! `funput-term` — type Vietnamese inside terminal apps via a transparent PTY wrapper.
//!
//! Run a program through it (`funput-term -- claude`) and ASCII keystrokes are
//! composed into Vietnamese before reaching the child; everything else is
//! forwarded untouched. Toggle with `Ctrl-\`. Not an IME — no system hooks, no
//! permissions; works in any terminal emulator.
//!
//! Settings come from the shared `Funput/settings.json`, overridable by env vars
//! and CLI flags (see [`config`]). `funput-term install` wires it into your shell.

mod app;
mod config;
mod inject;
mod input;
mod install;
mod output;
mod state;
mod term;

use clap::{Args, Parser, Subcommand, ValueEnum};
use funput_core::InputMethod;

#[derive(Parser)]
#[command(
    name = "funput-term",
    version,
    about = "Type Vietnamese (Telex/VNI) inside terminal apps via a PTY wrapper",
    args_conflicts_with_subcommands = true
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,

    #[command(flatten)]
    run: RunArgs,
}

/// Arguments for the default action: run a program through the wrapper.
#[derive(Args)]
struct RunArgs {
    /// Input method; overrides the config file and `$FUNPUT_METHOD`.
    #[arg(short, long, value_enum)]
    method: Option<Method>,

    /// Program to run (defaults to `$SHELL`). Pass after `--`, e.g. `funput-term -- claude`.
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    program: Vec<String>,
}

#[derive(Subcommand)]
enum Command {
    /// Print (or write) shell integration so funput-term is always on.
    Install {
        /// Target shell (bash/zsh/fish); defaults to `$SHELL`.
        #[arg(long)]
        shell: Option<String>,

        /// Alias to add, `name` or `name=command`; repeatable.
        #[arg(long = "alias", value_name = "NAME[=CMD]")]
        alias: Vec<String>,

        /// Append the snippet to your shell rc file instead of just printing it.
        #[arg(long)]
        write: bool,
    },
}

#[derive(Clone, Copy, ValueEnum)]
enum Method {
    Telex,
    Vni,
}

impl From<Method> for InputMethod {
    fn from(m: Method) -> Self {
        match m {
            Method::Telex => InputMethod::Telex,
            Method::Vni => InputMethod::Vni,
        }
    }
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Some(Command::Install {
            shell,
            alias,
            write,
        }) => install_cmd(shell, alias, write),
        None => run_cmd(cli.run),
    }
}

fn install_cmd(shell: Option<String>, alias: Vec<String>, write: bool) {
    let shell = shell
        .as_deref()
        .and_then(install::Shell::from_name)
        .unwrap_or_else(install::Shell::detect);
    let aliases: Vec<_> = alias.iter().map(|a| install::parse_alias(a)).collect();
    if let Err(err) = install::run(shell, &aliases, write) {
        eprintln!("funput-term: {err}");
        std::process::exit(1);
    }
}

fn run_cmd(args: RunArgs) {
    // Precedence: CLI flag > env var > settings.json > built-in default.
    let mut config = config::load();
    if let Some(method) = args.method {
        config.method = method.into();
    }

    let command = if args.program.is_empty() {
        vec![std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string())]
    } else {
        args.program
    };

    let opts = app::Options { config, command };

    match app::run(opts) {
        Ok(code) => std::process::exit(code),
        Err(err) => {
            eprintln!("funput-term: {err}");
            std::process::exit(1);
        }
    }
}
