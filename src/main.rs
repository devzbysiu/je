use crate::cfg::Cfg;
use crate::cmd::{Cmd, Opt};
use anyhow::Result;
use log::{debug, info};
use path::Path;
use std::env;
use structopt::StructOpt;

mod cfg;
mod cmd;
mod fsops;
mod path;
mod pkg;
mod pkgdir;
mod pkgmgr;

fn main() -> Result<()> {
    let opt = Opt::from_args();
    match opt.verbose {
        1 => {
            env::set_var("RUST_LOG", "je=info");
            info!("setting INFO log level");
        }
        2 => {
            env::set_var("RUST_LOG", "je=debug");
            info!("setting DEBUG log level");
        }
        _ => {}
    }
    pretty_env_logger::init();

    debug!("parsed opts: {:#?}", opt);
    info!("starting");
    let cfg = Cfg::load()?;
    debug!("read config: {:#?}", cfg);
    match opt.cmd {
        Cmd::Init => cmd::init()?,
        Cmd::Get { path } => cmd::get(opt.debug, &cfg, &Path::new(path))?,
        Cmd::Put { path } => cmd::put(opt.debug, &cfg, &Path::new(path))?,
    }
    Ok(())
}
