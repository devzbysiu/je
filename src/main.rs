use crate::cmd::Opt;
use anyhow::Result;
use log::{debug, info};
use std::env;
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

#[cfg(test)]
mod testutils;

fn main() -> Result<()> {
    let opt = Opt::from_args();
    setup_log_level_for_logger(&opt);
    pretty_env_logger::init();
    debug!("parsed opts: {:#?}", opt);
    debug!("current working dir: {:?}", env::current_dir());
    info!("starting");
    cmd::handle(&opt, &mut std::io::stdout())?;

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
