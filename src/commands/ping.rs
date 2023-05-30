use crate::{bot::Bot, util::DEFER_EPHEMERAL};
use std::{
	sync::Arc,
	time::{Duration, Instant},
};
use twilight::{
	model::application::{
		command::{Command, CommandType},
		interaction::{application_command::CommandData, Interaction},
	},
	util::builder::command::CommandBuilder,
};

pub const NAME: &str = "ping";

pub fn info() -> Command {
	CommandBuilder::new(NAME, "Check if the bot is alive", CommandType::ChatInput).build()
}

pub async fn run(bot: &Arc<Bot>, interaction: Interaction, _: CommandData) -> anyhow::Result<()> {
	let client = bot.interact();
	let defer = client.create_response(interaction.id, &interaction.token, &DEFER_EPHEMERAL);

	let time = Instant::now();
	let result = defer.await;
	let mut time = Instant::now() - time;
	result?;

	// only show millisecond precision
	time = Duration::from_millis(time.as_millis().try_into()?);

	client
		.update_response(&interaction.token)
		.content(Some(&format!("Pong! `{time:?}`")))?
		.await?;

	Ok(())
}
