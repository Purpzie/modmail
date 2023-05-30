use crate::{
	bot::Bot,
	util::{SqliteId, RED},
};
use anyhow::Context;
use sqlx::Row;
use std::sync::Arc;
use twilight::{
	id::MessageId, model::gateway::payload::incoming::MessageDelete,
	util::builder::embed::EmbedBuilder,
};

pub async fn handle(bot: Arc<Bot>, dm_msg_deleted: MessageDelete) -> anyhow::Result<()> {
	if dm_msg_deleted.guild_id.is_some() {
		return Ok(());
	}

	let ticket = match bot
		.db
		.ticket_by_dm_channel(dm_msg_deleted.channel_id)
		.await?
	{
		Some(ticket) if !ticket.is_open || ticket.blocked => return Ok(()),
		Some(ticket) => ticket,
		None => return Ok(()),
	};

	let thread_msg_id = sqlx::query(indoc! {"
		DELETE FROM messages
		WHERE user_id = ? AND dm_msg_id = ?
		RETURNING IFNULL(thread_update_msg_id, thread_msg_id)
	"})
	.bind(SqliteId(ticket.user_id))
	.bind(SqliteId(dm_msg_deleted.id))
	.try_map(|row| {
		let id: SqliteId<MessageId> = row.try_get(0)?;
		Ok(id.0)
	})
	.fetch_optional(&bot.db.connection)
	.await?
	.context("missing thread message id")?;

	let thread_msg = bot
		.http
		.message(ticket.thread_id, thread_msg_id)
		.await?
		.model()
		.await?;

	// the bot's messages can only be deleted with /delete, so staff already know about it.
	// don't send a duplicate "hey this message was deleted" notification
	if thread_msg.interaction.is_some() {
		return Ok(());
	}

	// include the deleted content instead of forcing staff to click the reply
	let mut embed = EmbedBuilder::new().color(RED).title("🗑️ Message deleted");
	if let Some(content) = thread_msg
		.embeds
		.into_iter()
		.next()
		.context("missing embed")?
		.description
	{
		embed = embed.description(content);
	}

	bot.http
		.create_message(ticket.thread_id)
		.reply(thread_msg_id)
		.embeds(&[embed.build()])?
		.await?;

	Ok(())
}
