use crate::{
	bot::Bot,
	util::{formatting, SqliteId, YELLOW},
};
use anyhow::Context;
use sqlx::Row;
use std::{sync::Arc, time::Duration};
use twilight::{
	http::request::channel::reaction::RequestReactionType,
	id::MessageId,
	model::{channel::message::MessageType, gateway::payload::incoming::MessageUpdate},
	util::builder::embed::{EmbedBuilder, EmbedFooterBuilder},
};

pub async fn handle(bot: Arc<Bot>, updated_msg: MessageUpdate) -> anyhow::Result<()> {
	let ignore = updated_msg.guild_id.is_some()
		|| updated_msg.kind.map_or(false, |kind| {
			!matches!(kind, MessageType::Regular | MessageType::Reply)
		});
	if ignore {
		return Ok(());
	}

	let user = match updated_msg.author {
		Some(author) if author.bot => return Ok(()),
		Some(author) => author,
		None => return Ok(()),
	};

	let content = match updated_msg.content {
		Some(string) if string.is_empty() => return Ok(()),
		Some(string) => string,
		None => return Ok(()),
	};

	let ticket = match bot.db.ticket_by_user(user.id).await? {
		Some(ticket) if !ticket.is_open || ticket.blocked => return Ok(()),
		Some(ticket) => ticket,
		None => return Ok(()),
	};

	let reply_to_thread_msg_id = sqlx::query(indoc! {"
		SELECT IFNULL(thread_update_msg_id, thread_msg_id) FROM messages
		WHERE user_id = ? AND dm_msg_id = ?
		ORDER BY rowid DESC
	"})
	.bind(SqliteId(user.id))
	.bind(SqliteId(updated_msg.id))
	.try_map(|row| {
		let id: SqliteId<MessageId> = row.try_get(0)?;
		Ok(id.0)
	})
	.fetch_optional(&bot.db.connection)
	.await?
	.context("missing data for edited message")?;

	let embed = EmbedBuilder::new()
		.color(YELLOW)
		.author(formatting::embed_author(user.id, &user.name, user.avatar))
		.description(content)
		.footer(EmbedFooterBuilder::new("✏️ Edited"))
		.build();

	let thread_update_msg = bot
		.http
		.create_message(ticket.thread_id)
		.reply(reply_to_thread_msg_id)
		.embeds(&[embed])?
		.await?
		.model()
		.await?;

	sqlx::query(indoc! {"
		UPDATE messages
		SET thread_update_msg_id = ?
		WHERE user_id = ? AND dm_msg_id = ?
	"})
	.bind(SqliteId(thread_update_msg.id))
	.bind(SqliteId(user.id))
	.bind(SqliteId(updated_msg.id))
	.execute(&bot.db.connection)
	.await?;

	// let the user know that their edit was sent
	static EDITED_REACTION: RequestReactionType = RequestReactionType::Unicode { name: "✏️" };
	bot.http
		.create_reaction(updated_msg.channel_id, updated_msg.id, &EDITED_REACTION)
		.await?;
	tokio::time::sleep(Duration::from_secs(2)).await;
	bot.http
		.delete_current_user_reaction(updated_msg.channel_id, updated_msg.id, &EDITED_REACTION)
		.await?;

	Ok(())
}
