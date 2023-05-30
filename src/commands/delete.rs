use crate::{
	bot::Bot,
	util::{InteractionResponseDataExt, SqliteId, DEFER, RED},
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
		embed::{EmbedBuilder, EmbedFieldBuilder, EmbedFooterBuilder},
		InteractionResponseDataBuilder,
	},
};

pub const NAME: &str = "delete";

pub fn info() -> Command {
	CommandBuilder::new(
		NAME,
		"Delete a message sent in this ticket",
		CommandType::ChatInput,
	)
	.default_member_permissions(Permissions::ADMINISTRATOR)
	.option(
		StringBuilder::new("id", "The ID of the message to delete")
			.required(true)
			.min_length(1)
			.max_length(20)
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

	// parse options
	let Some(thread_msg_id) = cmd_data.options.get(0).and_then(|opt| {
		if let CommandOptionValue::String(id_str) = &opt.value {
			MessageId::from_str(id_str).ok()
		} else {
			None
		}
	})
	else {
		bot.interact().create_response(
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

	// fetch the thread message they want to delete
	let thread_msg = match bot.http.message(thread_id, thread_msg_id).await {
		Ok(response) => response.model().await?,
		Err(_) => {
			bot.interact()
				.update_response(&interaction.token)
				.content(Some(
					"I couldn't find a message to delete. Did you copy the correct ID?",
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
			.content(Some("I can only delete messages I sent."))?
			.await?;
		return Ok(());
	}

	let ticket = bot
		.db
		.ticket_by_thread(thread_id)
		.await?
		.context("missing ticket")?;

	// we don't need to delete it from the database.
	// ../events/message_delete.rs already handles that for us
	let dm_msg_id = sqlx::query(indoc! {"
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

	// delete the dm
	bot.http
		.delete_message(ticket.dm_channel_id, dm_msg_id)
		.await?;

	// respond to the interaction
	let embed = EmbedBuilder::new()
		.color(RED)
		.description(format!(
			"🗑️ Deleted https://discord.com/channels/{}/{}/{}",
			bot.config.guild_id, thread_id, thread_msg_id,
		))
		.build();
	bot.interact()
		.update_response(&interaction.token)
		.embeds(Some(&[embed]))?
		.await?;

	// edit the corresponding thread message

	let mut old_embed: EmbedBuilder = thread_msg
		.embeds
		.into_iter()
		.next()
		.context("missing embed")?
		.into();

	old_embed = old_embed.color(RED);
	let deleting_author_id = interaction.author_id().context("missing author")?;
	let original_author_id = thread_msg
		.interaction
		.as_ref()
		.context("missing interaction")?
		.user
		.id;

	if deleting_author_id == original_author_id {
		old_embed = old_embed.footer(EmbedFooterBuilder::new("🗑️ Deleted"));
	} else {
		old_embed = old_embed.field(EmbedFieldBuilder::new(
			"🗑️ Deleted by",
			format!("<@{deleting_author_id}>"),
		));
	}

	bot.http
		.update_message(thread_id, thread_msg_id)
		.embeds(Some(&[old_embed.build()]))?
		.await?;

	Ok(())
}
