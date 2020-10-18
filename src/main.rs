use crate::cfg::Cfg;
use crate::cmd::{Cmd, Opt};
use anyhow::Result;
use args::{GetArgs, PutArgs};
use log::{debug, info};
use path::Path;
use std::env;
use structopt::StructOpt;

mod args;
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
    debug!("current workiong dir: {:?}", env::current_dir());
    info!("starting");
    match opt.cmd {
        Cmd::Init => cmd::init()?,
        other => {
            let cfg = Cfg::load()?;
            debug!("read config: {:#?}", cfg);
            match other {
                Cmd::Get { path } => {
                    let args = GetArgs {
                        path: Path::new(path),
                        instance: cfg.instance(opt.profile.as_ref()),
                        debug: opt.debug,
                        ignore_properties: cfg.ignore_properties,
                    };
                    cmd::get(args)?;
                }
                Cmd::Put { path } => {
                    let args = PutArgs {
                        path: Path::new(path),
                        instance: cfg.instance(opt.profile.as_ref()),
                        debug: opt.debug,
                    };
                    cmd::put(args)?
                }
                _ => unreachable!(),
            }
        }
    }
    Ok(())
}
