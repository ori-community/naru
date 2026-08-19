use crate::cli::CLI;
use crate::config::GameConfig;
use serenity::all::ChannelId;
use serenity::builder::{CreateButton, CreateMessage};
use serenity::futures::{StreamExt, TryStreamExt};
use serenity::http::Http;
use serenity::prelude::SerenityError;
use serenity::utils::MessageBuilder;
use speedrun_api::api::categories::Category;
use speedrun_api::api::games::{Game, GameId};
use speedrun_api::api::levels::Level;
use speedrun_api::api::runs::{RunStatus, Runs};
use speedrun_api::api::users::User;
use speedrun_api::api::{AsyncQuery, PagedEndpointExt};
use speedrun_api::error::{RestError, SpeedrunApiError};
use speedrun_api::types::{Player, Run};
use speedrun_api::{SpeedrunApiBuilder, SpeedrunApiClientAsync, types};
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tracing::info;

#[derive(PartialEq, Clone)]
struct HashableRun<'a>(Run<'a>);

impl<'a> Eq for HashableRun<'a> {}

impl<'a> Hash for HashableRun<'a> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.id.hash(state)
    }
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Generic speedrun.com API error")]
    SpeedrunApiError(#[from] SpeedrunApiError),
    #[error("Speedrun.com REST API error")]
    SpeedrunApiRestError(#[from] speedrun_api::api::ApiError<RestError>),
    #[error("Discord Error")]
    DiscordError(#[from] SerenityError),
}

async fn get_new_runs<'a>(
    game_id: GameId<'_>,
    known_runs: &HashSet<HashableRun<'a>>,
    client: &SpeedrunApiClientAsync,
) -> Result<HashSet<HashableRun<'a>>, SpeedrunApiError> {
    let new_runs_endpoint = Runs::builder()
        .game(game_id)
        .status(RunStatus::New)
        .build()
        .unwrap();

    let runs: HashSet<HashableRun> = new_runs_endpoint
        .stream::<Run, SpeedrunApiClientAsync>(&client)
        .take(100)
        .try_collect::<Vec<Run>>()
        .await?
        .into_iter()
        .map(|r| HashableRun(r))
        .collect();

    Ok(runs.difference(&known_runs).cloned().collect())
}

pub async fn speedrun_poll_loop(
    game_config: &GameConfig,
    discord_http: Arc<Http>,
) -> Result<(), Error> {
    let game_id = &game_config.speedrun_game_id;

    let client = SpeedrunApiBuilder::default()
        .api_key(&game_config.speedrun_api_token)
        .build_async()?;

    let game_endpoint = Game::builder().id(game_id).build().unwrap();
    let game: types::Game = game_endpoint.query_async(&client).await?;

    info!(
        "{game_id}: Resolved game id = {} ({})",
        game.names.international, game.abbreviation
    );

    let mut known_runs: HashSet<HashableRun> = HashSet::new();
    let wait_duration = Duration::from_mins(10);

    if !CLI.post_submissions_on_startup {
        known_runs = get_new_runs(game.id.clone(), &known_runs, &client).await?;

        info!(
            "{game_id}: Discovered {} pending run(s) on startup",
            known_runs.len()
        );

        tokio::time::sleep(wait_duration).await;
    }

    loop {
        let new_runs = get_new_runs(game.id.clone(), &known_runs, &client).await?;
        if !new_runs.is_empty() {
            info!("{game_id}: Discovered {} pending run(s)", new_runs.len());

            for HashableRun(run) in &new_runs {
                let category_endpoint = Category::builder()
                    .id(run.category.clone())
                    .build()
                    .unwrap();
                let category: types::Category = category_endpoint.query_async(&client).await?;

                let level_name = if let Some(level_id) = run.level.clone() {
                    let level_endpoint = Level::builder().id(level_id).build().unwrap();
                    let level: types::Level = level_endpoint.query_async(&client).await?;
                    Some(level.name)
                } else {
                    None
                };

                let mut player_names: Vec<String> = Vec::new();
                for player in &run.players {
                    player_names.push(match player {
                        Player::User { id, .. } => {
                            let player_endpoint = User::builder().id(id.clone()).build().unwrap();
                            let player: types::User = player_endpoint.query_async(&client).await?;
                            player.names.international
                        }
                        Player::Guest { name, .. } => name.clone(),
                    })
                }

                let message = CreateMessage::new()
                    .content(
                        MessageBuilder::new()
                            .push_line("**__New Submission__**")
                            .push_bold("Category: ")
                            .push_line_safe(format!(
                                "{}{} - {}",
                                game.names.international,
                                match level_name {
                                    None => "".to_string(),
                                    Some(level_name) => format!(" - {level_name}"),
                                },
                                category.name
                            ))
                            .push_bold("Runner: ")
                            .push_line_safe(player_names.join(", "))
                            .push_bold("Time: ")
                            .push_line_safe(
                                humantime::format_duration(Duration::from_secs_f64(
                                    run.times.primary_t,
                                ))
                                .to_string(),
                            )
                            .build(),
                    )
                    .button(CreateButton::new_link(run.weblink.clone()).label("View Submission"));

                discord_http
                    .send_message(
                        ChannelId::new(game_config.discord_channel_id),
                        vec![],
                        &message,
                    )
                    .await?;
            }

            known_runs.extend(new_runs);
        }

        tokio::time::sleep(wait_duration).await;
    }
}
