use anyhow::Result;
use clap::Subcommand;

pub mod list;

#[derive(Debug, Subcommand)]
pub enum Command {
    /// List connected Logitech HID++ devices.
    List(list::ListArgs),
}

impl Command {
    pub async fn run(self) -> Result<()> {
        match self {
            Self::List(args) => list::run(args).await,
        }
    }
}
