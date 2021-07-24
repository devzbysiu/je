use crate::cfg::Cfg;
use crate::cfgmgr::{Version, CONFIG_FILE};
use crate::cmd::{Cmd, Opt};
use anyhow::Result;
use args::{GetArgs, GetBundleArgs, PutArgs};
use log::{debug, info};
use std::env;
use std::fs::read_to_string;
use std::path::Path;
use structopt::StructOpt;

mod args;
mod cfg;
mod cfgmgr;
mod cmd;
mod fsops;
mod http;
mod path;
mod pkg;
mod pkgdir;
mod pkgmgr;

fn main() -> Result<()> {
    let opt = Opt::from_args();
    setup_log_level_for_logger(&opt);
    pretty_env_logger::init();
    debug!("parsed opts: {:#?}", opt);
    debug!("current working dir: {:?}", env::current_dir());
    info!("starting");
    handle_cmd(&opt)?;

    Ok(())
}

fn setup_log_level_for_logger(opt: &Opt) {
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
}

fn handle_cmd(opt: &Opt) -> Result<()> {
    match &opt.cmd {
        Cmd::Init => cmd::init(&Cfg::default())?,
        other => {
            if Path::new(CONFIG_FILE).exists() && *other != Cmd::Reinit {
                // if not Reinit, print warning message
                let version: Version = toml::from_str(&read_to_string(CONFIG_FILE)?)?;
                if version.value.is_none() {
                    // old, not versioned configuration
                    println!("###########################################");
                    println!("#                                         #");
                    println!("#    YOU ARE USING OLDER CONFIG FORMAT.   #");
                    println!("#    USE je reinit TO REINIT CONFIG       #");
                    println!("#                                         #");
                    println!("###########################################");
                }
            }
            let cfg = cfgmgr::handle_cfg_load()?;
            debug!("read config: {:#?}", cfg);
            match other {
                Cmd::Get { path } => cmd::get(&GetArgs::new(path, cfg, &opt))?,
                Cmd::GetBundle { name } => cmd::get_bundle(&GetBundleArgs::new(name, cfg, &opt))?,
                Cmd::Put { path } => cmd::put(&PutArgs::new(path, &cfg, &opt))?,
                Cmd::Reinit => cmd::init(&cfg)?,
                Cmd::Init => unreachable!("This code branch will never be executed"),
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_setup_logger_with_info_level() {
        // given
        let opt = Opt {
            verbose: 1,
            ..Opt::default()
        };
        env::set_var("RUST_LOG", "");

        // when
        setup_log_level_for_logger(&opt);

        // then
        assert_eq!(std::env::var("RUST_LOG").unwrap(), "je=info");
    }

    #[test]
    fn test_setup_logger_with_debug_level() {
        // given
        let opt = Opt {
            verbose: 2,
            ..Opt::default()
        };
        env::set_var("RUST_LOG", "");

        // when
        setup_log_level_for_logger(&opt);

        // then
        assert_eq!(std::env::var("RUST_LOG").unwrap(), "je=debug");
    }

    #[test]
    fn test_setup_logger_without_log_level() {
        // given
        let opt = Opt {
            verbose: 0,
            ..Opt::default()
        };
        env::set_var("RUST_LOG", "");

        // when
        setup_log_level_for_logger(&opt);

        // then
        assert_eq!(std::env::var("RUST_LOG").unwrap(), "");
    }
}
