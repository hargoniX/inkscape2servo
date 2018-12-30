#![feature(proc_macro_hygiene, decl_macro)]
use quicli::prelude::*;
use structopt::StructOpt;

mod serve;

#[derive(StructOpt, Debug)]
#[structopt(
    name = "grbl-servo-server",
    about = "receive gcodes from grbl-servo-client and push the to rabbitmq"
)]
struct Cli {
    /// IP to serve on
    ip: String,
    /// Port to serve on
    #[structopt(short = "p", long = "port")]
    port: Option<u16>,
}

fn main() -> CliResult {
    let args = Cli::from_args();
    serve::serve(args.ip, args.port);
    Ok(())
}