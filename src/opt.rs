use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "je",
    about = "Jcr Exchange - easy download and upload files to and from JCR"
)]
pub(crate) enum Opt {
    /// Download server content to local file server
    Get {
        /// path to download
        path: String,
    },
    Init,
}
