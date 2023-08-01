use clap::Parser;
use event_extractor::{self, config::Config, process_entry};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Configuration file
    #[arg(short, long)]
    config: String,
}

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let args = Args::parse();
    let config = Config::from_file(&args.config)?;

    for entry in config.entries {
        match process_entry(&entry) {
            Ok(_) => {}
            Err(e) => log::error!("{}", e),
        }
    }

    Ok(())
}
