use clap::Parser;
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
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            if args.verbose {
                "ippi=debug"
            } else {
                "ippi=info"
            }
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();
    
    tracing::info!("Starting IPPI v{}", env!("CARGO_PKG_VERSION"));
    tracing::info!("Configuration file: {}", args.config);
    
    // Simulate server startup
    tracing::info!("Web server would start on 0.0.0.0:8080");
    tracing::info!("Press Ctrl+C to exit");
    
    // Wait for shutdown signal
    tokio::signal::ctrl_c().await?;
    tracing::info!("Shutting down...");
    
    Ok(())
}