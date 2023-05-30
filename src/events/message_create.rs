use crate::{
	bot::Bot,
	util::{formatting, SqliteId, BLANK_EMBED_COLOR},
};
use sqlx::Row;
use std::{fmt::Write as _, sync::Arc};
use twilight::{
	http::request::channel::reaction::RequestReactionType,
	id::MessageId,
	model::{
		channel::message::{MessageReference, MessageType},
		gateway::payload::incoming::MessageCreate,
	},
	util::builder::embed::{EmbedBuilder, ImageSource},
};

pub async fn handle(bot: Arc<Bot>, dm_msg: MessageCreate) -> anyhow::Result<()> {
	// removing this makes rust complain about ownership?
	let mut dm_msg = dm_msg.0;

	if dm_msg.author.bot
		|| dm_msg.guild_id.is_some()
		|| !matches!(dm_msg.kind, MessageType::Regular | MessageType::Reply)
	{
		return Ok(());
	}

	let mut ticket = match bot.db.ticket_by_user(dm_msg.author.id).await? {
		Some(ticket) if ticket.blocked => return Ok(()),
		Some(ticket) => ticket,
		None => bot.create_ticket(dm_msg.author.id).await?,
	};

	if !ticket.is_open {
		bot.open_ticket(&mut ticket, true).await?;
	}

	// build the embed we'll send to the thread
	let mut embed = EmbedBuilder::new()
		.color(BLANK_EMBED_COLOR)
		.author(formatting::embed_author(
			dm_msg.author.id,
			&dm_msg.author.name,
			dm_msg.author.avatar,
		));

	if let Some(sticker) = dm_msg.sticker_items.get(0) {
		write!(dm_msg.content, "\n[Sticker: {}]", sticker.name)?;
		if let Some(url) = formatting::sticker_url(sticker) {
			embed = embed.image(ImageSource::url(url)?);
		}
	}

	if !dm_msg.content.is_empty() {
		embed = embed.description(dm_msg.content);
	}

	let embed = embed.build();

	let mut thread_msg = bot.http.create_message(ticket.thread_id);

	// if the user replied to a message, reply to the corresponding one in the thread
	if let Some(MessageReference {
		message_id: Some(replied_dm_msg_id),
		..
	}) = dm_msg.reference
	{
		let reply_to_thread_msg_id = sqlx::query(indoc! {"
			SELECT thread_msg_id FROM messages
			WHERE user_id = ? AND dm_msg_id = ?
		"})
		.bind(SqliteId(dm_msg.author.id))
		.bind(SqliteId(replied_dm_msg_id))
		.try_map(|row| {
			let id: SqliteId<MessageId> = row.try_get(0)?;
			Ok(*id)
		})
		.fetch_optional(&bot.db.connection)
		.await?;
		if let Some(reply_to_thread_msg_id) = reply_to_thread_msg_id {
			thread_msg = thread_msg.reply(reply_to_thread_msg_id);
		}
	}

	// send it to the thread
	let thread_msg = thread_msg.embeds(&[embed])?.await?.model().await?;

	// link to attachments
	if !dm_msg.attachments.is_empty() {
		let mut attachment_text = String::from("Attachments:");
		for attachment in &dm_msg.attachments {
			attachment_text.push('\n');
			attachment_text.push_str(&attachment.url);
		}
		bot.http
			.create_message(ticket.thread_id)
			.content(&attachment_text)?
			.await?;
	}

	// let the user know that it was sent
	if let Err(err) = bot
		.http
		.create_reaction(
			dm_msg.channel_id,
			dm_msg.id,
			&RequestReactionType::Unicode { name: "📨" },
		)
		.await
	{
		error!(?err);
	}

	// save ids
	sqlx::query(indoc! {"
		INSERT INTO messages (user_id, dm_msg_id, thread_msg_id)
		VALUES (?, ?, ?)
	"})
	.bind(SqliteId(dm_msg.author.id))
	.bind(SqliteId(dm_msg.id))
	.bind(SqliteId(thread_msg.id))
	.execute(&bot.db.connection)
	.await?;

	Ok(())
}
