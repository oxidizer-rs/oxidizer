use clap::Clap;

use super::errors::*;

#[derive(Clap)]
pub struct Init {
    path: Option<String>,
    overwrite: Option<bool>
}

pub fn init(opts: Init) -> Result<i32, Error> {

    let path = match opts.path {
        Some(p) => p,
        None => std::env::current_dir()?.to_str().ok_or(Error::Other("Could not get current dir".to_string()))?.to_string(),
    };

    let overwrite = opts.overwrite.unwrap_or(false);



    Ok(0)
}