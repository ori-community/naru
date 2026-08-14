use serenity::all::{Context, EventHandler, Guild, Ready};
use serenity::async_trait;
use tracing::info;
pub struct MessageHandler;

impl MessageHandler {
    pub fn new() -> Self {
        MessageHandler {}
    }
}

#[async_trait]
impl EventHandler for MessageHandler {
    async fn guild_create(&self, _ctx: Context, guild: Guild, _is_new: Option<bool>) {
        info!("Discovered Guild {} ({})", guild.name, guild.id);
    }

    async fn ready(&self, _ctx: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);
    }
}
