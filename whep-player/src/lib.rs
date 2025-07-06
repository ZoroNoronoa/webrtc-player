use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "bitwhip")]
#[command(bin_name = "bitwhip")]
pub struct Cli {
    #[command(subcommand)]
    pub commands: Commands,

    /// Increase log verbosity, multiple occurrences (-vvv) further increase
    #[clap(short, global = true, action = clap::ArgAction::Count)]
    pub verbose: u8,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Stream to a WHIP destination
    #[command(arg_required_else_help = true)]
    Stream {
        /// The WHIP URL
        url: String,

        /// The WHIP bearer token
        token: Option<String>,
    },

    /// Start a WHIP server that accepts incoming requests
    PlayWHIP {},

    /// Play from a WHEP destination
    #[command(arg_required_else_help = true)]
    PlayWHEP {
        /// The WHEP URL
        url: String,

        /// The WHEP bearer token
        token: Option<String>,
    },
}

pub mod util;
