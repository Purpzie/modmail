use crate::{bot::Bot, util::InteractionResponseDataExt};
use std::sync::Arc;
use twilight::{
	model::{
		application::{
			command::{Command, CommandType},
			interaction::{application_command::CommandData, Interaction},
		},
		channel::message::MessageFlags,
	},
	util::builder::{command::CommandBuilder, InteractionResponseDataBuilder},
};

pub const NAME: &str = "about";

pub fn info() -> Command {
	CommandBuilder::new(
		NAME,
		"View information about the bot",
		CommandType::ChatInput,
	)
	.build()
}

pub async fn run(bot: &Arc<Bot>, interaction: Interaction, _: CommandData) -> anyhow::Result<()> {
	bot.interact()
		.create_response(
			interaction.id,
			&interaction.token,
			&InteractionResponseDataBuilder::new()
				.flags(MessageFlags::EPHEMERAL | MessageFlags::SUPPRESS_EMBEDS)
				.content(concatdoc! {"
					**I'm an open-source modmail bot written in Rust.** 🦀
					Version: `", env!("CARGO_PKG_VERSION"), "`
					Source code: [Purpzie/modmail](https://github.com/purpzie/modmail) (MIT)
					Discord library: [Twilight](https://github.com/twilight-rs/twilight)
				"})
				.into_response(),
		)
		.await?;

	Ok(())
}
