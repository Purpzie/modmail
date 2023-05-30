use crate::{
	bot::Bot,
	util::{formatting, DEFER},
};
use anyhow::Context;
use std::sync::Arc;
use twilight::{
	model::{
		application::{
			command::{Command, CommandType},
			interaction::{application_command::CommandData, Interaction},
		},
		guild::Permissions,
	},
	util::builder::command::CommandBuilder,
};

pub const NAME: &str = "info";

pub fn info() -> Command {
	CommandBuilder::new(
		NAME,
		"Get information about the user in this modmail thread",
		CommandType::ChatInput,
	)
	.default_member_permissions(Permissions::ADMINISTRATOR)
	.build()
}

pub async fn run(bot: &Arc<Bot>, interaction: Interaction, _: CommandData) -> anyhow::Result<()> {
	let Some(thread_id) = super::only_in_modmail_thread(bot, &interaction).await?
	else {
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

	let info_embed = formatting::user_info_embed(bot, ticket.user_id).await?;

	bot.interact()
		.update_response(&interaction.token)
		.content(Some(&format!("<@{}>", ticket.user_id)))?
		.embeds(Some(&[info_embed]))?
		.await?;

	Ok(())
}
