use std::{borrow::Cow, path::PathBuf, sync::Arc};

use clap::{Parser, Subcommand};

pub type SafeMetaInfo = Arc<MetaInfo>;

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    Serve,
}

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct MetaInfo {
    /// Hostname to host server.
    #[arg(long, default_value_t = Cow::Borrowed("0.0.0.0"))]
    pub host: Cow<'static, str>,

    /// Port to host server.
    #[arg(short, long, default_value_t = 6688)]
    pub port: u16,

    /// Server data save path. Include database files.
    #[arg(short, long)]
    pub data_path: PathBuf,

        #[command(subcommand)]
    pub cmd: Commands,
}
impl MetaInfo {
    pub fn new() -> SafeMetaInfo {
        let meta = MetaInfo::parse();
        
        Arc::new(meta)
    }
}
