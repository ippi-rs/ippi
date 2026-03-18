use clap::Parser;
use ippi::{Config, Result};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "config/ippi.toml")]
    config: String,

    #[arg(short, long)]
    verbose: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(if args.verbose {
            "ippi=debug"
        } else {
            "ippi=info"
        }))
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting {} v{}", ippi::NAME, ippi::VERSION);

    let config = Config::load(&args.config).await?;
    tracing::debug!("Loaded configuration: {:?}", config);

    ippi::web::serve(config).await?;

    Ok(())
}
