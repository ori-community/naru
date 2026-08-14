use clap::Parser;
use std::sync::LazyLock;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct CliArgs {
    /// Path to config file
    #[arg(short, long, default_value = "config.yaml")]
    pub config: String,

    /// Post pending submissions on bot startup
    #[arg(short, default_value = "false")]
    pub post_submissions_on_startup: bool,
}

pub static CLI: LazyLock<CliArgs> = LazyLock::new(|| CliArgs::parse());
