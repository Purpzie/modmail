use crate::{bot::Bot, util::DEFER};
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
	util::builder::command::{BooleanBuilder, CommandBuilder},
};

pub const NAME: &str = "close";

pub fn info() -> Command {
	CommandBuilder::new(NAME, "Close this modmail ticket", CommandType::ChatInput)
		.default_member_permissions(Permissions::ADMINISTRATOR)
		.option(BooleanBuilder::new(
			"silent",
			"Whether to close the ticket without sending the user a message",
		))
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
	let silent = match cmd_data.options.into_iter().next().map(|opt| opt.value) {
		Some(CommandOptionValue::Boolean(value)) => value,
		_ => false,
	};

	bot.interact()
		.create_response(interaction.id, &interaction.token, &DEFER)
		.await?;

	let mut ticket = bot
		.db
		.ticket_by_thread(thread_id)
		.await?
		.context("missing ticket")?;

	if let Err(err) = bot
		.interact()
		.update_response(&interaction.token)
		.content(Some(if silent {
			"Closing silently..."
		} else {
			"Closing..."
		}))?
		.await
	{
		error!(?err);
	}

	bot.close_ticket(&mut ticket, !silent).await?;

	Ok(())
}
