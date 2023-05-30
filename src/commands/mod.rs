use crate::{bot::Bot, util::InteractionResponseDataExt};
use anyhow::Context;
use std::sync::Arc;
use twilight::{
	id::ChannelId,
	model::{
		application::{
			command::Command,
			interaction::{application_command::CommandData, Interaction},
		},
		channel::message::MessageFlags,
	},
	util::builder::InteractionResponseDataBuilder,
};

const VALID_SENDING_COMMANDS: &[&str] = &["reply", "tag"];

async fn only_in_modmail_thread(
	bot: &Arc<Bot>,
	interaction: &Interaction,
) -> anyhow::Result<Option<ChannelId>> {
	let thread = interaction.channel.as_ref().context("missing channel")?;
	if thread.parent_id != Some(bot.config.forum_channel_id) {
		bot.interact()
			.create_response(
				interaction.id,
				&interaction.token,
				&InteractionResponseDataBuilder::new()
					.flags(MessageFlags::EPHEMERAL)
					.content("You can only use this command in a modmail thread.")
					.into_response(),
			)
			.await?;

		Ok(None)
	} else {
		Ok(Some(thread.id))
	}
}

macro_rules! commands {
	($($mod_name:ident),*$(,)?) => {
		pub fn info() -> Vec<Command> {
			vec![$($mod_name::info()),*]
		}

		pub async fn handle_command(
			bot: &Arc<Bot>,
			interaction: Interaction,
			data: CommandData,
		) -> anyhow::Result<()> {
			match &data.name as &str {
				$($mod_name::NAME => $mod_name::run(bot, interaction, data).await),*,
				_ => Ok(())
			}
		}
	};
}

// modules need to be outside of macros for rustfmt to find them
mod about;
mod close;
mod delete;
mod edit;
mod info;
mod link;
mod modmail;
mod ping;
mod reply;

commands! {
	about,
	close,
	delete,
	edit,
	info,
	link,
	modmail,
	ping,
	reply,
}
