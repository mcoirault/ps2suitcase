use chrono::NaiveDateTime;
use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug, Clone)]
#[command(version, about = "", arg_required_else_help(true))]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) command: Commands,
}

#[derive(Subcommand, Debug, Clone)]
pub(crate) enum Commands {
    /// Create a .psu from scratch
    Create(CreateArgs),
    /// Read the content of a psu
    Read(ReadArgs),
    /// Provide a .toml file to automate the creation of multiple .psu files
    Automate(AutomateArgs),
}

#[derive(Args, Debug, Clone)]
pub(crate) struct CreateArgs {
    #[arg(value_name = "FILES", help = "One or many files to add to the psu")]
    pub(crate) files: Vec<String>,

    #[arg(
        short = 'n',
        long,
        value_name = "STRING",
        help = "Name of the psu folder"
    )]
    pub(crate) name: String,

    #[arg(
        short = 'o',
        long,
        value_name = "PATH",
        help = "Output path, uses {name}.psu by default"
    )]
    pub(crate) output: Option<String>,

    #[arg(
        short = 't',
        long,
        value_name = "PATH",
        help = "The timestamp to be applied to files in the psu"
    )]
    pub(crate) timestamp: Option<NaiveDateTime>,
}

#[derive(Args, Debug, Clone)]
pub(crate) struct ReadArgs {
    #[arg(value_name = "FILE", help = "Path of the psu to read")]
    pub(crate) file: String,
}

#[derive(Args, Debug, Clone)]
pub(crate) struct AutomateArgs {
    #[arg(value_name = "FILE", help = "Path of the .toml to use")]
    pub(crate) toml: String,

    #[arg(short = 'o', long, help = "If this flag is provided, any existing .psu will be overwritten")]
    pub(crate) overwrite: bool,
}
