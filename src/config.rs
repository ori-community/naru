use crate::cli::CLI;
use config::Config;
use serde::Deserialize;
use std::sync::LazyLock;

#[derive(Deserialize)]
pub struct GameConfig {
    pub speedrun_api_token: String,
    pub speedrun_game_id: String,
    pub discord_channel_id: u64,
}

#[derive(Deserialize)]
pub struct AppConfig {
    pub discord_bot_token: String,
    pub games: Vec<GameConfig>,
}

pub static CONFIG: LazyLock<AppConfig> = LazyLock::new(|| {
    Config::builder()
        .add_source(config::File::with_name(&CLI.config))
        .add_source(config::Environment::with_prefix("NARU"))
        .build()
        .unwrap()
        .try_deserialize::<AppConfig>()
        .expect("Unable to parse config")
});
