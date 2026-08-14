mod cli;
mod config;
mod discord_bot;
mod speedrun_poll;

use crate::config::CONFIG;
use crate::discord_bot::MessageHandler;
use serenity::Client;
use serenity::all::GatewayIntents;
use tokio::task::JoinSet;
use tracing::{error, log};

#[tokio::main]
async fn main() {
    env_logger::builder()
        .filter_module("naru", log::LevelFilter::Info)
        .init();

    let mut discord_client = Client::builder(&CONFIG.discord_bot_token, GatewayIntents::GUILDS)
        .event_handler(MessageHandler::new())
        .await
        .expect("Error creating client");

    let discord_http = discord_client.http.clone();

    let mut tasks = JoinSet::new();

    tasks.spawn(async move {
        if let Err(err) = discord_client.start().await {
            error!("Discord client error: {err}");
        }
    });

    for game_config in &CONFIG.games {
        tasks.spawn({
            let discord_http = discord_http.clone();

            async move {
                if let Err(err) =
                    speedrun_poll::speedrun_poll_loop(&game_config, discord_http).await
                {
                    error!("Error while polling speedrun API: {err}")
                }
            }
        });
    }

    // Abort everything when a task finishes because all tasks
    // should never finish without an error.
    tasks.join_next().await;
    tasks.shutdown().await;
}
