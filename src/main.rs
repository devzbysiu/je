use crate::cfg::Cfg;
use crate::cmd::Opt;
use anyhow::Result;
use log::debug;
use path::Path;
use structopt::StructOpt;

mod cfg;
mod cmd;
mod path;
mod pkg;
mod pkgdir;
mod pkgmgr;

fn main() -> Result<()> {
    pretty_env_logger::init();
    let opt = Opt::from_args();
    debug!("parsed opts: {:#?}", opt);
    let cfg = Cfg::load()?;
    debug!("read config: {:#?}", cfg);
    match opt {
        Opt::Get { path } => cmd::get(&cfg, Path::new(path))?,
        Opt::Init => cmd::init()?,
    }
    Ok(())
}
