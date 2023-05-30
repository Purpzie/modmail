use crate::bot::Bot;
use std::{sync::Arc, time::Duration};
use twilight::gateway::Event;

mod guild_create;
mod interaction_create;
mod message_create;
mod message_delete;
mod message_update;
mod new_audit_log_entry;
mod ready;

pub async fn handle_event(bot: Arc<Bot>, event: Event) -> anyhow::Result<()> {
	bot.discord_cache.update(&event);

	match event {
		Event::InteractionCreate(interaction) => {
			interaction_create::handle(bot, *interaction).await
		},
		Event::MessageCreate(msg) => message_create::handle(bot, *msg).await?,
		Event::MessageUpdate(msg) => message_update::handle(bot, *msg).await?,
		Event::MessageDelete(msg) => message_delete::handle(bot, msg).await?,
		Event::Ready(_) => ready::handle(bot)?,
		Event::GuildCreate(guild) => guild_create::handle(bot, *guild)?,
		Event::GuildAuditLogEntryCreate(entry) => new_audit_log_entry::handle(bot, *entry).await?,

		Event::ThreadUpdate(channel) => {
			if channel.parent_id != Some(bot.config.forum_channel_id) {
				return Ok(());
			}
			let Some(metadata) = &channel.thread_metadata else {
				return Ok(());
			};
			let Some(ticket) = bot.db.ticket_by_thread(channel.id).await? else {
				return Ok(());
			};

			if ticket.is_open && (metadata.archived || metadata.locked) {
				warn!("reopening thread {}", channel.id);
				bot.http
					.update_thread(channel.id)
					.archived(false)
					.locked(false)
					.await?;
			}
		},

		Event::ThreadDelete(thread) => {
			if thread.parent_id != bot.config.forum_channel_id {
				return Ok(());
			}
			warn!("thread {} deleted", thread.id);
			bot.db.delete_ticket(thread.id).await?;
		},

		Event::MemberRemove(info) => {
			let ticket = match bot.db.ticket_by_user(info.user.id).await? {
				None => return Ok(()),
				Some(ticket) if !ticket.is_open || ticket.blocked => return Ok(()),
				Some(ticket) => ticket,
			};
			bot.http
				.create_message(ticket.thread_id)
				.content("👋 User left the server.")?
				.await?;
		},

		Event::MemberAdd(member) => {
			let ticket = match bot.db.ticket_by_user(member.user.id).await? {
				None => return Ok(()),
				Some(ticket) if !ticket.is_open || ticket.blocked => return Ok(()),
				Some(ticket) => ticket,
			};
			bot.http
				.create_message(ticket.thread_id)
				.content("👋 User rejoined the server.")?
				.await?;
		},

		Event::GatewayClose(close_frame) => warn!(?close_frame, "gateway closed"),
		Event::GatewayHello(hello) => {
			debug!(heartbeat = ?Duration::from_millis(hello.heartbeat_interval), "gateway hello")
		},
		Event::GatewayInvalidateSession(resumable) => {
			warn!(resumable, "gateway invalidated session")
		},
		Event::GatewayReconnect => warn!("gateway indicates reconnect"),
		Event::Resumed => info!("resumed"),
		Event::UnavailableGuild(g) => warn!(id = g.id.get(), "unavailable guild"),
		Event::GuildDelete(g) => warn!(id = g.id.get(), "left guild"),
		Event::MemberChunk(c) => {
			debug!(
				"received member chunk {}/{}",
				c.chunk_index + 1,
				c.chunk_count
			)
		},

		_ => (),
	};

	Ok(())
}
