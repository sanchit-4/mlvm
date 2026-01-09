mod cli;
mod commands;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Language, NodeCommand, PythonCommand};

// Our main function needs to be async because our network request is async.
// The `#[tokio::main]` macro sets up the async runtime for us.
#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // The first level of dispatching: which language was chosen?
    match cli.language {
        Language::Node { command } => {
            // The second level of dispatching: which command for that language?
            match command {
                NodeCommand::ListRemote => {
                    commands::node::list_remote().await?;
                }
                NodeCommand::Install { version } => { // <-- Add this arm
                    commands::node::install(&version).await?;
                }
                NodeCommand::Use { version } => { // <-- Add this arm
                    commands::node::use_version(&version)?;
                }
                NodeCommand::List => { // <-- Add this arm
                    commands::node::list_local()?;
                }
            }
        }

        Language::Python { command } => {
            match command {
                PythonCommand::ListRemote => {
                    // We will create this function next
                    commands::python::list_remote().await?;
                }
                PythonCommand::Install { version } => { // <-- Add this arm
                    commands::python::install(&version).await?;
                }
            }
        }
    }

    Ok(())
}