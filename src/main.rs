use crate::cfg::Cfg;
use crate::cmd::{Cmd, Opt};
use anyhow::Result;
use log::{debug, info};
use path::Path;
use std::env;
use structopt::StructOpt;

mod cfg;
mod cmd;
mod path;
mod pkg;
mod pkgdir;
mod pkgmgr;

fn main() -> Result<()> {
    let opt = Opt::from_args();
    if opt.verbose {
        env::set_var("RUST_LOG", "je=info");
        info!("setting INFO level");
    }
    pretty_env_logger::init();

    debug!("parsed opts: {:#?}", opt);
    info!("starting");
    let cfg = Cfg::load()?;
    debug!("read config: {:#?}", cfg);
    match opt.cmd {
        Cmd::Get { path } => cmd::get(&cfg, Path::new(path))?,
        Cmd::Init => cmd::init()?,
    }
    Ok(())
}
