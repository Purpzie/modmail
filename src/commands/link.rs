use crate::{
	bot::Bot,
	util::{InteractionResponseDataExt, SqliteId, DEFER},
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
		InteractionResponseDataBuilder,
	},
};

pub const NAME: &str = "link";

pub fn info() -> Command {
	CommandBuilder::new(
		NAME,
		"Get a message link from this modmail thread (useful for reporting)",
		CommandType::ChatInput,
	)
	.default_member_permissions(Permissions::ADMINISTRATOR)
	.option(
		StringBuilder::new("id", "The thread message ID to get a link for")
			.min_length(1)
			.max_length(20)
			.required(true)
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

	// parse argument
	let Some(thread_msg_id) = (match cmd_data.options.get(0).map(|opt| &opt.value) {
		Some(CommandOptionValue::String(s)) => MessageId::from_str(s).ok(),
		_ => None,
	}) else {
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

	let ticket = bot
		.db
		.ticket_by_thread(thread_id)
		.await?
		.context("missing ticket")?;

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
	.await?;

	let Some(dm_msg_id) = dm_msg_id
	else {
		bot.interact().update_response(&interaction.token)
			.content(Some("I couldn't find that message in the database. Did you copy the correct ID?"))?
			.await?;
		return Ok(());
	};

	bot.interact()
		.update_response(&interaction.token)
		.content(Some(&formatdoc! {"
			https://discord.com/channels/@me/{}/{}
			^ Right-click (or hold on mobile) to copy that as a link.",
			ticket.dm_channel_id,
			dm_msg_id,
		}))?
		.await?;

	Ok(())
}
