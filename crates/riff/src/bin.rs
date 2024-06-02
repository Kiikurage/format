use clap::Parser;
use riff::riff::Chunk;

#[derive(Parser, Debug)]
#[command(about)]
/// Parse RIFF file structure
struct Args {
    /// Input file path
    #[command()]
    path_str: String,
}

fn main() -> Result<(), std::io::Error> {
    let args = Args::parse();
    let chunk = Chunk::open(args.path_str)?;

    chunk.print();

    Ok(())
}
