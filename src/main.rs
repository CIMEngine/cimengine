use clap::Parser;

mod build;
mod init;
mod types;

use types::Commands;

fn main() {
    let args = types::Cli::parse();

    match args.cmd {
        Commands::Build => build::build(),
        Commands::Init { name } => init::init(name),
        _ => unimplemented!(),
    }
}
