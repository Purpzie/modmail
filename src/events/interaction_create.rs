use crate::{bot::Bot, commands};
use std::sync::Arc;
use twilight::model::{
	application::interaction::InteractionData, gateway::payload::incoming::InteractionCreate,
};

pub async fn handle(bot: Arc<Bot>, mut interaction: InteractionCreate) {
	let Some(InteractionData::ApplicationCommand(data)) = interaction.data.take() else {
		return;
	};

	// TODO: add better information to this
	if let Err(err) = commands::handle_command(&bot, interaction.0, *data).await {
		error!(?err);
	}
}
