#![feature(proc_macro_hygiene, decl_macro)]

use quicli::prelude::*;
use structopt::StructOpt;

mod process;
mod upload;

#[derive(StructOpt, Debug)]
#[structopt(
    name = "grbl-servo-client",
    about = "Process and upload gcode files, generated by inkscape, for a grbl servo fork"
)]
struct Cli {
    #[structopt(subcommand)]
    subcommand: Subcommands,
}

#[derive(StructOpt, Debug)]
enum Subcommands {
    #[structopt(name = "process")]
    /// Process the given gcode file
    Process {
        /// File to process
        file: String,
        /// Output processed gcode to file instead of printing to stdout
        #[structopt(short = "o", long = "outfile")]
        outfile: Option<String>,
    },

    #[structopt(name = "upload")]
    /// Upload the given gcode file
    Upload {
        /// File to upload
        file: String,
        /// IP of the server to upload to
        #[structopt(short = "i", long = "ip")]
        ip: String,
        /// Port of the server to upload to
        #[structopt(short = "p", long = "port")]
        port: Option<String>,
    },

}

fn main() -> CliResult {
    let args = Cli::from_args();
    match args.subcommand {
        Subcommands::Process { file, outfile } => process::process(file, outfile),
        Subcommands::Upload { file, ip, port } => upload::upload(file, ip, port),
    }
    Ok(())
}