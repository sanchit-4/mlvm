// mod cli;
// mod commands;

// use anyhow::Result;
// use clap::Parser;
// use cli::{Cli, Language, NodeCommand, PythonCommand};

// // Our main function needs to be async because our network request is async.
// // The `#[tokio::main]` macro sets up the async runtime for us.
// #[tokio::main]
// async fn main() -> Result<()> {
//     let cli = Cli::parse();

//     // The first level of dispatching: which language was chosen?
//     match cli.language {
//         Language::Node { command } => {
//             // The second level of dispatching: which command for that language?
//             match command {
//                 NodeCommand::ListRemote => {
//                     commands::node::list_remote().await?;
//                 }
//                 NodeCommand::Install { version } => { // <-- Add this arm
//                     commands::node::install(&version).await?;
//                 }
//                 NodeCommand::Use { version } => { // <-- Add this arm
//                     commands::node::use_version(&version)?;
//                 }
//                 NodeCommand::List => { // <-- Add this arm
//                     commands::node::list_local()?;
//                 }
//             }
//         }

//         // Inside main.rs -> match command
//         Language::Python { command } => {
//             match command {
//                 PythonCommand::ListRemote=>commands::python::list_remote().await?,
//                 PythonCommand::Install{version}=>commands::python::install(&version).await?,
//                 PythonCommand::Use { version } => todo!(),
//                 PythonCommand::List => todo!(),
//             }
//         }

//         Language::Go { command } => {
//             match command {
//                 cli::GoCommand::ListRemote => commands::go::list_remote().await?,
//                 cli::GoCommand::Install { version } => commands::go::install(&version).await?,
//                 cli::GoCommand::Use { version } => commands::go::use_version(&version)?,
//             }
//         }
//     }

//     Ok(())
// }


mod cli;
mod commands;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Language, NodeCommand, PythonCommand, GoCommand, BunCommand};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.language {
        Language::Node { command } => match command {
            NodeCommand::ListRemote => commands::node::list_remote().await?,
            NodeCommand::Install { version } => commands::node::install(&version).await?,
            NodeCommand::Use { version } => commands::node::use_version(&version)?,
            NodeCommand::List => commands::node::list_local()?,
        },
        Language::Python { command } => match command {
            PythonCommand::ListRemote=>commands::python::list_remote().await?,
            PythonCommand::Install{version}=>commands::python::install(&version).await?,
            PythonCommand::Use{version}=>commands::python::use_version(&version)?,
            PythonCommand::List => todo!(),
                    },
        Language::Go { command } => match command {
            GoCommand::ListRemote => commands::go::list_remote().await?,
            GoCommand::Install { version } => commands::go::install(&version).await?,
            GoCommand::Use { version } => commands::go::use_version(&version)?,
        },
        Language::Bun { command } => match command {
            BunCommand::ListRemote => commands::bun::list_remote().await?,
            BunCommand::Install { version } => commands::bun::install(&version).await?,
            BunCommand::Use { version } => commands::bun::use_version(&version)?,
        },
    }

    Ok(())
}