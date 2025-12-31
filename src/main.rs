use std::env;

use dotenvy::dotenv;

use serenity::async_trait;
use serenity::builder::CreateMessage;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use serenity::utils::MessageBuilder;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "!ping" {
            let channel = match msg.channel_id.to_channel(&ctx).await {
                Ok(channel) => channel,
                Err(e) => {
                    println!("Error getting channel: {e:?}");
                    return;
                }
            };

            let response = MessageBuilder::new()
                .push("User ")
                .push_bold_safe(&msg.author.name)
                .push(" used the 'ping' command in the ")
                .mention(&channel)
                .push(" channel")
                .build();

            if let Err(e) = msg.channel_id.say(&ctx.http, &response).await {
                println!("Error sending message: {}", e);
            }
        }

        if msg.content == "!messageme" {
            let builder = CreateMessage::new().content("Hello!");
            let dm = msg.author.dm(&ctx, builder).await;

            if let Err(e) = dm {
                println!("Error when direct messaging user: {e:?}");
            }
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        if let Some(shard) = ready.shard {
            println!("{} is connected on shard {}/{}!", ready.user.name, shard.id, shard.total);
        }
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let token = env::var("DISCORD_TOKEN")
        .expect("Expected a token in the environment");

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;
    
    let mut client =
        Client::builder(&token, intents).event_handler(Handler).await.expect("Err creating client");

    if let Err(e) = client.start_shards(2).await {
        println!("Client error: {e:?}");
    }
}
