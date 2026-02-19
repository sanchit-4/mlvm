use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(author, version, about = "A Multi Language Version Manager", long_about = None)]
pub struct Cli {
#[command(subcommand)]
pub language: Language,
}

#[derive(Subcommand, Debug)]
pub enum Language {
/// Commands for managing Node.js versions
    Node {
    #[command(subcommand)]
    command: NodeCommand,
    },

    Python { // <-- Add this new variant
        #[command(subcommand)]
        command: PythonCommand,
    },

    Go { 
        #[command(subcommand)]
        command: GoCommand },

    Bun { #[command(subcommand)] command: BunCommand },    
// We can easily add Go { #[command(subcommand)] command: GoCommand } here later!
}

#[derive(Subcommand, Debug)]
pub enum NodeCommand {
    /// List all available remote versions of Node.js
    ListRemote,
    /// Install a specific version of Node.js
    Install {
        /// The version to install (e.g., "18.17.1" or "v18.17.1")
        version: String,
    },
    Use {
        /// The version to use (e.g., "18.17.1")
        version: String,
    },
    List,
}

#[derive(Subcommand, Debug)]
pub enum PythonCommand {
    /// List all available remote versions of Python
    ListRemote,
    /// Install a specific version of Python
    Install { // <-- Add this
        /// The version to install (e.g., "3.11.5")
        version: String,
    },
    Use { version: String }, // Add this
    List
}

#[derive(Subcommand, Debug)]
pub enum GoCommand {
    ListRemote,
    Install { version: String },
    Use { version: String },
}

#[derive(Subcommand, Debug)]
pub enum BunCommand {
    ListRemote,
    Install { version: String },
    Use { version: String },
}