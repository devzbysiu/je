use anyhow::Result;
use log::debug;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "je",
    about = "Jcr Exchange - easy download and upload files to and from JCR"
)]
enum Opt {
    /// Download server content to local file server
    Get {
        /// path to download
        path: String,
    },
}

fn main() -> Result<()> {
    pretty_env_logger::init();
    let opt = Opt::from_args();
    debug!("parsed opts: {:#?}", opt);
    match opt {
        Opt::Get { path } => get(path)?,
    }
    Ok(())
}

fn get<S: Into<String>>(path: S) -> Result<()> {
    debug!("downloading path {}", path.into());
    Ok(())
}
