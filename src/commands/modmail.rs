use crate::{bot::Bot, util::DEFER_EPHEMERAL};
use anyhow::Context;
use std::sync::Arc;
use twilight::{
	model::application::{
		command::{Command, CommandType},
		interaction::{application_command::CommandData, Interaction},
	},
	util::builder::command::CommandBuilder,
};

pub const NAME: &str = "modmail";

pub fn info() -> Command {
	CommandBuilder::new(NAME, "Open a modmail ticket", CommandType::ChatInput).build()
}

pub async fn run(bot: &Arc<Bot>, interaction: Interaction, _: CommandData) -> anyhow::Result<()> {
	let user_id = interaction.author_id().context("missing author id")?;

	bot.interact()
		.create_response(interaction.id, &interaction.token, &DEFER_EPHEMERAL)
		.await?;

	let send_error_msg = || async {
		bot.interact()
			.update_response(&interaction.token)
			.content(Some(indoc! {"
				⚠️ **I couldn't send you a message.**
				Make sure that you're accepting DMs from this server and that you haven't blocked me.
			"}))?
			.await?;
		Ok::<_, anyhow::Error>(())
	};

	let dm_channel = match bot.http.create_private_channel(user_id).await {
		Ok(response) => response.model().await?,
		Err(_) => {
			send_error_msg().await?;
			return Ok(());
		},
	};

	let dm_msg = match bot
		.http
		.create_message(dm_channel.id)
		.content("Send a message here to open a ticket.")?
		.await
	{
		Ok(response) => response.model().await?,
		Err(_) => {
			send_error_msg().await?;
			return Ok(());
		},
	};

	bot.interact()
		.update_response(&interaction.token)
		.content(Some(&format!(
			"Click here: https://discord.com/channels/@me/{}/{}",
			dm_channel.id, dm_msg.id,
		)))?
		.await?;

	Ok(())
}
