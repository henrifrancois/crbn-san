use crate::{Context, Error};
use poise::serenity_prelude as serenity;
use serenity::Mentionable;
use songbird::events::{Event, EventContext, TrackEvent, EventHandler as VoiceEventHandler};
use reqwest::Client as HttpClient;
use songbird::input::YoutubeDl;

struct TrackErrorNotifier;

#[serenity::async_trait]
impl VoiceEventHandler for TrackErrorNotifier {
    async fn act(&self, ctx: &EventContext<'_>) -> Option<Event> {
        if let EventContext::Track(track_list) = ctx {
            for (state, handle) in *track_list {
                println!(
                    "Track {:?} encountered an error: {:?}",
                    handle.uuid(),
                    state.playing
                );
            }
        }
        None
    }
}

/// Join the voice channel you are in
#[poise::command(slash_command, guild_only)]
pub async fn join(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer().await?;

    let (guild_id, channel_id) = {
        let guild = ctx.guild().ok_or("Context doesn't contain a guild")?;
        let channel_id = guild
            .voice_states
            .get(&ctx.author().id)
            .and_then(|voice_state| voice_state.channel_id);
        (guild.id, channel_id)
    };

    let connect_to = match channel_id {
        Some(channel) => channel,
        None => {
            ctx.say("You need to be in a voice channel").await?;
            return Ok(());
        }
    };

    // Scope to fetch members and drop the guild reference immediately
    let members_in_channel: Vec<_> = {
        let guild = ctx.guild().ok_or("Could not get guild")?;
        guild
            .voice_states
            .values()
            .filter(|vs| vs.channel_id == Some(connect_to))
            .map(|vs| vs.user_id)
            .collect()
    };

    if members_in_channel.is_empty() {
        ctx.say("Voice channel is empty.").await?;
        return Ok(());
    }

    let mut admin_present = false;
    let current_user_id = ctx.serenity_context().cache.current_user().id;

    for user_id in members_in_channel {
        if user_id == current_user_id {
            continue;
        }

        if let Ok(member) = ctx.http().get_member(guild_id, user_id).await {
            if let Some(guild) = ctx.guild() {
                if let Some(channel) = guild.channels.get(&connect_to) {
                    let permissions = guild.user_permissions_in(channel, &member);
                    if permissions.contains(serenity::Permissions::ADMINISTRATOR) {
                        admin_present = true;
                        break;
                    }
                }
            }
        }
    }

    if !admin_present {
        ctx
            .say("I can only join if there is an admin present in the voice channel.")
            .await?;
        return Ok(());
    }

    let manager = songbird::get(ctx.serenity_context())
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    // Check if we are already in the guild and leave to ensure clean state
    if manager.get(guild_id).is_some() {
        let _ = manager.leave(guild_id).await;
    }

    match manager.join(guild_id, connect_to).await {
        Ok(handler_lock) => {
            // Attach an event handler to see notifications of all track errors.
            let mut handler = handler_lock.lock().await;
            handler.add_global_event(TrackEvent::Error.into(), TrackErrorNotifier);
            drop(handler); // Release the lock
            
            ctx.say(format!("Joined {}", connect_to.mention())).await?;
        }
        Err(e) => {
            ctx.say(format!("Failed to join voice channel: {:?}", e)).await?;
        }
    }

    Ok(())
}

#[poise::command(slash_command, guild_only)]
pub async fn leave(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer().await?;

    let guild_id = ctx.guild_id().unwrap();

    let manager = songbird::get(ctx.serenity_context())
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();
    let has_handler = manager.get(guild_id).is_some();

    if has_handler {
        if let Err(e) = manager.remove(guild_id).await {
            ctx.say(format!("Failed to leave voice channel: {:?}", e)).await?;
        } else {
            ctx.say("Left voice channel").await?;
        }
    } else {
        ctx.say("I am not in a voice channel").await?;
    }

    Ok(())
}

/// Play audio from a URL
#[poise::command(slash_command, guild_only)]
pub async fn play(
    ctx: Context<'_>,
    #[description = "URL to play"] url: String,
) -> Result<(), Error> {
    ctx.defer().await?;

    let guild_id = ctx.guild_id().ok_or("Not in a guild")?;

    // Determine target channel (User's channel)
    let channel_id = ctx.guild().unwrap()
        .voice_states.get(&ctx.author().id)
        .and_then(|vs| vs.channel_id);

    let connect_to = match channel_id {
        Some(channel) => channel,
        None => {
            ctx.say("You need to be in a voice channel to play music.").await?;
            return Ok(());
        }
    };

    let manager = songbird::get(ctx.serenity_context())
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone();

    // Ensure connection
    let handler_lock = if let Some(call) = manager.get(guild_id) {
        call
    } else {
        match manager.join(guild_id, connect_to).await {
            Ok(call) => call,
            Err(e) => {
                ctx.say(format!("Failed to join voice channel: {:?}", e)).await?;
                return Ok(());
            }
        }
    };

    let mut handler = handler_lock.lock().await;

    // Use yt-dlp to source audio
    let source = YoutubeDl::new(HttpClient::new(), url);
    let _handle = handler.play_input(source.into());

    ctx.say("Playing...").await?;

    Ok(())
}
