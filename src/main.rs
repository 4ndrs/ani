mod anilist;
mod cli;
mod completion;
mod display;
mod models;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    cli::parse_args().await?;

    Ok(())
}
