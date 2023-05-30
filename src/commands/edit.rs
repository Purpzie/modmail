use crate::{
	bot::Bot,
	util::{InteractionResponseDataExt, SqliteId, DEFER, GREEN, YELLOW},
};
use anyhow::Context;
use sqlx::Row;
use std::{str::FromStr, sync::Arc};
use twilight::{
	id::MessageId,
	model::{
		application::{
			command::{Command, CommandType},
			interaction::{
				application_command::{CommandData, CommandOptionValue},
				Interaction,
			},
		},
		channel::message::MessageFlags,
		guild::Permissions,
	},
	util::builder::{
		command::{CommandBuilder, StringBuilder},
		embed::{EmbedBuilder, EmbedFieldBuilder},
		InteractionResponseDataBuilder,
	},
	validate::message::MESSAGE_CONTENT_LENGTH_MAX,
};

pub const NAME: &str = "edit";

pub fn info() -> Command {
	CommandBuilder::new(
		NAME,
		"Edit a message sent in this ticket",
		CommandType::ChatInput,
	)
	.default_member_permissions(Permissions::ADMINISTRATOR)
	.option(
		StringBuilder::new("id", "The ID of the message to edit")
			.required(true)
			.min_length(1)
			.max_length(20)
			.build(),
	)
	.option(
		StringBuilder::new("to", "What to edit the message to")
			.required(true)
			.min_length(1)
			.max_length(MESSAGE_CONTENT_LENGTH_MAX as u16)
			.build(),
	)
	.build()
}

pub async fn run(
	bot: &Arc<Bot>,
	interaction: Interaction,
	cmd_data: CommandData,
) -> anyhow::Result<()> {
	let Some(thread_id) = super::only_in_modmail_thread(bot, &interaction).await?
	else {
		return Ok(());
	};

	// parse arguments
	let mut thread_msg_id = None;
	let mut new_content = None;
	for opt in cmd_data.options.iter() {
		match (&opt.name as &str, &opt.value) {
			("id", CommandOptionValue::String(s)) => thread_msg_id = MessageId::from_str(s).ok(),
			("to", CommandOptionValue::String(s)) => new_content = Some(s),
			_ => (),
		}
	}
	let new_content = new_content.context("missing new text arg")?;
	let Some(thread_msg_id) = thread_msg_id
	else {
		bot.interact()
			.create_response(
				interaction.id,
				&interaction.token,
				&InteractionResponseDataBuilder::new()
					.content("Please specify a valid message ID.")
					.flags(MessageFlags::EPHEMERAL)
					.into_response()
			)
			.await?;
		return Ok(());
	};

	bot.interact()
		.create_response(interaction.id, &interaction.token, &DEFER)
		.await?;

	// fetch the thread message that they want to edit
	let thread_msg = match bot.http.message(thread_id, thread_msg_id).await {
		Ok(response) => response.model().await?,
		Err(_) => {
			bot.interact()
				.update_response(&interaction.token)
				.content(Some(
					"I couldn't find a message to edit. Did you copy the correct ID?",
				))?
				.await?;
			return Ok(());
		},
	};

	// make sure we can actually use it
	if thread_msg.author.id != bot.user_id
		|| thread_msg.interaction.as_ref().map_or(true, |i| {
			!super::VALID_SENDING_COMMANDS.iter().any(|&n| i.name == n)
		}) {
		bot.interact()
			.update_response(&interaction.token)
			.content(Some("I can only edit messages I sent."))?
			.await?;
		return Ok(());
	}

	let ticket = bot
		.db
		.ticket_by_thread(thread_id)
		.await?
		.context("missing ticket")?;

	let editing_dm_msg_id = sqlx::query(indoc! {"
		SELECT dm_msg_id FROM messages
		WHERE user_id = ? AND thread_msg_id = ?
		ORDER BY rowid DESC
	"})
	.bind(SqliteId(ticket.user_id))
	.bind(SqliteId(thread_msg_id))
	.try_map(|row| {
		let id: SqliteId<MessageId> = row.try_get(0)?;
		Ok(*id)
	})
	.fetch_optional(&bot.db.connection)
	.await?
	.context("missing dm_msg_id")?;

	// edit the dm
	bot.http
		.update_message(ticket.dm_channel_id, editing_dm_msg_id)
		.content(Some(new_content))?
		.await?;

	let mut old_embed = thread_msg
		.embeds
		.into_iter()
		.next()
		.context("missing embed")?;

	let old_content = old_embed
		.description
		.as_ref()
		.context("missing description")?;

	// respond to the interaction
	let embed = EmbedBuilder::new()
		.color(YELLOW)
		.description(format!(
			"✏️ Edited https://discord.com/channels/{}/{}/{}",
			bot.config.forum_guild_id, thread_id, thread_msg_id,
		))
		.field(EmbedFieldBuilder::new("Before", old_content).build())
		.field(EmbedFieldBuilder::new("After", new_content).build())
		.build();
	bot.interact()
		.update_response(&interaction.token)
		.embeds(Some(&[embed]))?
		.await?;

	// edit the corresponding thread message

	old_embed.fields.clear(); // get rid of existing "edited by" field if it's there
	let mut old_embed = EmbedBuilder::from(old_embed)
		.color(GREEN)
		.description(new_content);

	let editing_author_id = interaction.author_id().context("missing author")?;
	let original_author_id = thread_msg
		.interaction
		.as_ref()
		.context("missing interaction")?
		.user
		.id;

	if editing_author_id != original_author_id {
		old_embed = old_embed.field(EmbedFieldBuilder::new(
			"✏️ Edited by",
			format!("<@{editing_author_id}>"),
		));
	}

	bot.http
		.update_message(thread_id, thread_msg_id)
		.embeds(Some(&[old_embed.build()]))?
		.await?;

	Ok(())
}
