use crate::bot::Bot;
use std::sync::{
	atomic::{AtomicBool, Ordering},
	Arc,
};
use twilight::model::gateway::payload::outgoing::RequestGuildMembers;

static STARTUP_RAN: AtomicBool = AtomicBool::new(false);

pub fn handle(bot: Arc<Bot>) -> anyhow::Result<()> {
	info!("ready");

	if STARTUP_RAN.load(Ordering::Acquire) {
		return Ok(());
	}
	STARTUP_RAN.store(true, Ordering::Release);

	let result = bot
		.discord_websocket
		.command(&RequestGuildMembers::builder(bot.config.guild_id).query("", None));
	if let Err(err) = result {
		error!(?err, "error requesting guild members");
		bot.stop();
	}

	Ok(())
}
