use clap::Parser;

mod build;
mod init;
mod new;
mod types;
mod utils;

use types::Commands;

fn main() {
    let args = types::Cli::parse();

    match args.cmd {
        Commands::Build => build::build(),
        Commands::Init { name } => init::init(name),
        Commands::New { cmd } => new::new(cmd),
        _ => unimplemented!(),
    }
}
