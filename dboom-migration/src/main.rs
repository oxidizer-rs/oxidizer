use clap::Clap;
use std::process;

mod errors;
use errors::*;

mod helpers;

mod init;
use init::*;

/// This doc string acts as a help message when the user runs '--help'
/// as do all doc strings on fields
#[derive(Clap)]
#[clap(version = "1.0", author = "Gustavo Sampaio <gbritosampaio@gmail.com>")]
struct Opts {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Clap)]
enum SubCommand {
    #[clap(version = "1.0", author = "Gustavo Sampaio <gbritosampaio@gmail.com>")]
    Init(Init),
}

fn main() {
    let opts: Opts = Opts::parse();

    // You can handle information about subcommands by requesting their matches by name
    // (as below), requesting just the name used, or both at the same time
    let res = match opts.subcmd {
        SubCommand::Init(t) => {
            init(t)
        }
    };

    match res {
        Ok(r) => process::exit(r),
        Err(err) => panic!(err),
    }
}