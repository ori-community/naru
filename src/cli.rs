use std::sync::LazyLock;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct CliArgs {
    /// Path to config file
    #[arg(short, long, default_value = "config.yaml")]
    pub config: String,
}

pub static CLI: LazyLock<CliArgs> = LazyLock::new(|| {
    CliArgs::parse()
});