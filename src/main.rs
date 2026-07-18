use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(version)]
/// Some anime cli
struct Cli {
    #[command(subcommand)]
    commands: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Show information about an anime
    Info {
        /// The AniList anime id
        id: u32,
    },
}

fn main() {
    let args = Cli::parse();

    match args.commands {
        Commands::Info { id } => {
            println!("Show info for id {}", id)
        }
    }
}
