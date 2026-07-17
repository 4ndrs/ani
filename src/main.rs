use clap::Parser;

#[derive(Parser, Debug)]
#[command(version)]
struct Args {
    #[arg(short, long, default_value = "ANON TOKYO")]
    name: String,
}

fn main() {
    let args = Args::parse();

    println!("Hello, {}!", args.name);
}
