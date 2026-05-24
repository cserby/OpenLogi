use anyhow::Result;
use clap::Parser;
use tracing_subscriber::{EnvFilter, fmt};

mod cmd;

/// Options-minus: a local-first companion for Logitech HID++ peripherals.
#[derive(Debug, Parser)]
#[command(
    name = "optminus",
    version,
    about = "Options-minus: a local-first companion for Logitech HID++ peripherals.",
    long_about = None,
)]
struct Cli {
    #[command(subcommand)]
    cmd: Option<cmd::Command>,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(
            EnvFilter::try_from_env("OPTMINUS_LOG").unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let cli = Cli::parse();
    let command = cli
        .cmd
        .unwrap_or(cmd::Command::List(cmd::list::ListArgs {}));
    command.run().await
}
