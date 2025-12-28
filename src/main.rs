#![warn(clippy::str_to_string)]
#[allow(dead_code)]

mod commands;
mod tts;

use dotenv::from_filename;
use poise::serenity_prelude as serenity;
use songbird::SerenityInit;
use reqwest::Client as HttpClient;
use crate::tts::TTSMessageData;


use std::{
    env::var,
    sync::Arc,
    time::Duration,
};

// Types used by all command functions
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

struct HttpKey;

impl serenity::prelude::TypeMapKey for HttpKey {
    type Value = HttpClient;
}


// Custom user data passed to all command functions
pub struct Data {}

async fn on_error(error: poise::FrameworkError<'_, Data, Error>) {
    // This is our custom error handler
    match error {
        poise::FrameworkError::Setup { error, .. } => panic!("Failed to start bot: {:?}", error),
        poise::FrameworkError::Command { error, ctx, .. } => {
            println!("Error in command `{}`: {:?}", ctx.command().name, error,);
        }
        error => {
            if let Err(e) = poise::builtins::on_error(error).await {
                println!("Error while handling error: {}", e)
            }
        }
    }
}

async fn check_authorization(_ctx: &serenity::Context, msg: &serenity::Message) -> bool {
    // TODO: Implement actual database check for authorized channels
    // For now, we allow all channels if the user is not a bot.
    // In a real implementation, we would check:
    // 1. Is this channel in the list of authorized TTS channels for this guild?
    // 2. Does the user have the required role (if applicable)?
    
    if msg.author.bot {
        return false;
    }

    // Example permission check (placeholder)
    // if let Some(member) = msg.member(&ctx).await.ok() {
    //     // Check roles...
    // }

    true
}

async fn event_handler(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    _data: &Data,
) -> Result<(), Error> {
    match event {
        serenity::FullEvent::Ready { data_about_bot, .. } => {
            println!("Logged in as {}", data_about_bot.user.name);
        }
        serenity::FullEvent::Message { new_message } => {
            if check_authorization(ctx, new_message).await {
                let endpoint = std::env::var("TTS_API_ENDPOINT").unwrap_or_default();
                if endpoint.is_empty() {
                    // Fail silently or log if no endpoint configured
                    return Ok(());
                }

                let data = {
                    let client_data = ctx.data.read().await;
                    client_data.get::<HttpKey>().cloned()
                };

                if let Some(client) = data {
                    let tts_data = TTSMessageData {
                        guild_id: new_message.guild_id.map(|id| id.to_string()),
                        channel_id: new_message.channel_id.to_string(),
                        user_id: new_message.author.id.to_string(),
                        username: new_message.author.name.clone(),
                        display_name: new_message.author_nick(&ctx).await.unwrap_or_else(|| new_message.author.name.clone()),
                        message_content: new_message.content.clone(),
                        timestamp: new_message.timestamp.to_rfc3339().unwrap_or_default(),
                        message_id: new_message.id.to_string(),
                        voice: None, // TODO: Fetch voice preference from DB
                    };

                    // Send POST request with JSON payload
                    match client.post(&endpoint).json(&tts_data).send().await {
                        Ok(resp) => {
                            if !resp.status().is_success() {
                                println!("TTS API error: {}", resp.status());
                            }
                        },
                        Err(e) => println!("Failed to send TTS request: {}", e),
                    }
                }
            }
        }
        _ => {}
    }
    Ok(())
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    from_filename(".env.local").ok();

    // FrameworkOptions contains all of poise's configuration option in one struct
    // Every option can be omitted to use its default value
    let options = poise::FrameworkOptions {
        commands: vec![
            commands::help::help(),
            commands::tts::tts(),
            commands::voice::join(),
            commands::voice::leave(),
            commands::voice::play(),
            commands::register::register(),
        ],
        prefix_options: poise::PrefixFrameworkOptions {
            prefix: Some("~".into()),
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
        // This code is run before every command
        pre_command: |ctx| {
            Box::pin(async move {
                println!("Executing command {}...", ctx.command().qualified_name);
            })
        },
        // This code is run after a command if it was successful (returned Ok)
        post_command: |ctx| {
            Box::pin(async move {
                println!("Executed command {}!", ctx.command().qualified_name);
            })
        },
        // Every command invocation must pass this check to continue execution
        command_check: Some(|ctx| {
            Box::pin(async move {
                if ctx.author().id == 123456789 {
                    return Ok(false);
                }
                Ok(true)
            })
        }),
        // Enforce command checks even for owners (enforced by default)
        // Set to true to bypass checks, which is useful for testing
        skip_checks_for_owners: true,
        event_handler: |ctx, event, framework, data| {
            Box::pin(event_handler(ctx, event, framework, data))
        },
        ..Default::default()
    };

    let token = var("DISCORD_TOKEN")
        .expect("Missing `DISCORD_TOKEN` env var, see README for more information.");

    let framework = poise::Framework::builder()
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                println!("Logged in as {}", _ready.user.name);
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {})
            })
        })
        .options(options)
        .build();

    let intents = serenity::GatewayIntents::non_privileged()
        | serenity::GatewayIntents::MESSAGE_CONTENT
        | serenity::GatewayIntents::GUILD_VOICE_STATES
        | serenity::GatewayIntents::GUILDS;

    let mut client = serenity::ClientBuilder::new(&token, intents)
        .framework(framework)
        .register_songbird()
        .type_map_insert::<HttpKey>(HttpClient::new())
        .await
        .expect("Error creating client");

    tokio::spawn(async move {
        let _ = client
            .start()
            .await
            .map_err(|why| println!("Client ended: {:?}", why));
    });

    let _signal_err = tokio::signal::ctrl_c().await;
    println!("Received Ctrl-C, shutting down.");

}
