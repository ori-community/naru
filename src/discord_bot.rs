use serenity::all::{Context, EventHandler, Guild, GuildId, Ready};
use serenity::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::info;

#[derive(Clone)]
struct GuildInfo {
    name: String,
}

pub struct MessageHandler {
    guilds: Arc<Mutex<HashMap<GuildId, GuildInfo>>>,
}

impl MessageHandler {
    pub fn new() -> Self {
        MessageHandler {
            guilds: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl EventHandler for MessageHandler {
    async fn guild_create(&self, _ctx: Context, guild: Guild, _is_new: Option<bool>) {
        info!("Discovered Guild {} ({})", guild.name, guild.id);
        self.guilds.lock().await.insert(
            guild.id,
            GuildInfo {
                name: guild.name.clone(),
            },
        );
    }

    async fn ready(&self, _ctx: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);
    }
}
