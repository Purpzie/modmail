use crate::{
	bot::Bot,
	util::{SqliteId, DEFER, GREEN},
};
use anyhow::Context;
use std::sync::Arc;
use twilight::{
	model::{
		application::{
			command::{Command, CommandType},
			interaction::{
				application_command::{CommandData, CommandOptionValue},
				Interaction,
			},
		},
		guild::Permissions,
	},
	util::builder::{
		command::{CommandBuilder, StringBuilder},
		embed::EmbedBuilder,
	},
	validate::message::MESSAGE_CONTENT_LENGTH_MAX,
};

pub const NAME: &str = "reply";

pub fn info() -> Command {
	CommandBuilder::new(NAME, "Reply to a modmail ticket", CommandType::ChatInput)
		.default_member_permissions(Permissions::ADMINISTRATOR)
		.option(
			StringBuilder::new("with", "The text to reply with")
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

	// get argument
	let text = cmd_data
		.options
		.get(0)
		.and_then(|option| {
			if let CommandOptionValue::String(text) = &option.value {
				Some(text)
			} else {
				None
			}
		})
		.context("missing argument")?;

	bot.interact()
		.create_response(interaction.id, &interaction.token, &DEFER)
		.await?;

	let ticket = bot
		.db
		.ticket_by_thread(thread_id)
		.await?
		.context("missing ticket")?;

	// send the dm
	let dm_msg_id = match bot
		.http
		.create_message(ticket.dm_channel_id)
		.content(text)?
		.await
	{
		Ok(response) => response.model().await?.id,
		Err(_) => {
			bot.interact()
				.update_response(&interaction.token)
				.content(Some(
					"⚠️ Unable to send DM. The user may have DMs closed or they blocked me.",
				))?
				.await?;
			return Ok(());
		},
	};

	// respond to the interaction
	let embed = EmbedBuilder::new().color(GREEN).description(text).build();
	if let Err(err) = bot
		.interact()
		.update_response(&interaction.token)
		.embeds(Some(&[embed]))?
		.await
	{
		error!(?err);
	}

	// save message ids

	let response_msg_id = bot
		.interact()
		.response(&interaction.token)
		.await?
		.model()
		.await?
		.id;

	sqlx::query(indoc! {"
		INSERT INTO messages (user_id, dm_msg_id, thread_msg_id)
		VALUES (?, ?, ?)
	"})
	.bind(SqliteId(ticket.user_id))
	.bind(SqliteId(dm_msg_id))
	.bind(SqliteId(response_msg_id))
	.execute(&bot.db.connection)
	.await?;

	Ok(())
}
