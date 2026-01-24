mod alerts;
mod commands;
mod config;
mod db;
use config::load_config;
use db::Database;

use dotenvy::dotenv;
use poise::serenity_prelude as serenity;
use std::{
    collections::HashMap,
    env::var,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::Mutex;
use tracing::{error, info, warn};

// Types used by all command functions
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

// Custom user data passed to all command functions
pub struct Data {
    votes: Mutex<HashMap<String, u32>>,
    start_time: Instant,
    db: Mutex<Database>,
    db_path: String,
}

async fn on_error(error: poise::FrameworkError<'_, Data, Error>) {
    // This is our custom error handler
    // They are many errors that can occur, so we only handle the ones we want to customize
    // and forward the rest to the default handler
    match error {
        poise::FrameworkError::Setup { error, .. } => panic!("Failed to start bot: {:?}", error),
        poise::FrameworkError::Command { error, ctx, .. } => {
            error!("Error in command `{}`: {:?}", ctx.command().name, error,);
        }
        error => {
            if let Err(e) = poise::builtins::on_error(error).await {
                error!("Error while handling error: {}", e)
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    dotenv().ok();

    // FrameworkOptions contains all of poise's configuration option in one struct
    // Every option can be omitted to use its default value
    let options = poise::FrameworkOptions {
        commands: vec![
            commands::fun::ban(),
            commands::fun::boop(),
            commands::fun::coinflip(),
            commands::fun::ferrisparty(),
            commands::fun::immuwune(),
            commands::fun::mock(),
            commands::fun::mock_ctx_menu(),
            commands::fun::owo(),
            commands::fun::ping(),
            commands::fun::roll(),
            commands::fun::say(),
            commands::fun::trickortreat(),
            commands::fun::uwu(),
            commands::fun::wiki(),
            commands::fun::yeet(),

            commands::info::echo(),
            commands::info::getvotes(),
            commands::info::help(),
            commands::info::joined(),
            commands::info::prse(),
            commands::info::shutdown(),
            commands::info::uptime(),
            commands::info::vote(),

            commands::roles::add(),
            commands::roles::create_roles(),
            commands::roles::del(),
            commands::roles::list_roles(),
            commands::roles::my_roles(),
        ],
        prefix_options: poise::PrefixFrameworkOptions {
            prefix: Some("!".into()),
            edit_tracker: Some(Arc::new(poise::EditTracker::for_timespan(
                Duration::from_secs(3600),
            ))),
            additional_prefixes: vec![
                poise::Prefix::Literal("hey bot,"),
                poise::Prefix::Literal("hey bot"),
            ],
            ..Default::default()
        },
        // The global error handler for all error cases that may occur
        on_error: |error| Box::pin(on_error(error)),
        // Enforce command checks even for owners (enforced by default)
        // Set to true to bypass checks, which is useful for testing
        skip_checks_for_owners: false,
        // This code is run before every command
        #[cfg(debug_assertions)] 
        pre_command: |ctx| {
            Box::pin(async move {
                println!("Executing command {}...", ctx.command().qualified_name);
            })
        },
        // This code is run after a command if it was successful (returned Ok)
        #[cfg(debug_assertions)] 
        post_command: |ctx| {
            Box::pin(async move {
                println!("Executed command {}!", ctx.command().qualified_name);
            })
        },
        #[cfg(debug_assertions)] 
        event_handler: |_ctx, event, _framework, _data| {
            Box::pin(async move {
                println!(
                    "Got an event in event handler: {:?}",
                    event.snake_case_name()
                );
                Ok(())
            })
        },
        ..Default::default()
    };

    let framework = poise::Framework::builder()
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                info!("Logged in as {}", _ready.user.name);
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;

                match load_config() {
                    // Only spawn alerts task if the config was provided & parsed correctly
                    Ok(config) => {
                        info!("Loaded config.toml");
                        let http = ctx.http.clone();
                        let (tx, rx) = tokio::sync::watch::channel(config.clone());
                        tokio::spawn(async move {
                            let _ = alerts::alerts(http, config, rx).await;
                        });
                        tokio::spawn(async move {
                            let _ = config::watch_config(tx).await;
                        });
                    },
                    Err(e) => warn!("Running without NWS alerts due to toml error: {e}"),
                }

                let db_path = String::from("db.json");
                let db = Mutex::new(
                    Database::load(&db_path).await?,
                );

                Ok(Data {
                    votes: Mutex::new(HashMap::new()),
                    start_time: Instant::now(),
                    db,
                    db_path,
                })
            })
        })
        .options(options)
        .build();

    let token = var("DISCORD_TOKEN")
        .expect("Missing `DISCORD_TOKEN` env var, see README for more information.");
    let intents =
        serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT;

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;

    client?.start().await?;

    Ok(())
}
