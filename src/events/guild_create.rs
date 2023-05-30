use crate::bot::Bot;
use std::sync::Arc;
use twilight::model::{gateway::payload::incoming::GuildCreate, guild::Permissions};

pub fn handle(bot: Arc<Bot>, guild: GuildCreate) -> anyhow::Result<()> {
	info!(name = ?guild.name, id = guild.id.get(), "joined guild");

	if guild.id == bot.config.forum_guild_id {
		let current_perms = bot
			.discord_cache
			.permissions()
			.in_channel(bot.user_id, bot.config.forum_channel_id)?;

		let required_perms = Permissions::VIEW_CHANNEL
			| Permissions::SEND_MESSAGES
			| Permissions::SEND_MESSAGES_IN_THREADS
			| Permissions::MANAGE_THREADS
			| Permissions::EMBED_LINKS
			| Permissions::READ_MESSAGE_HISTORY;

		let missing_perms = required_perms - current_perms;
		if !missing_perms.is_empty() {
			error!("BOT IS MISSING REQUIRED PERMISSIONS IN FORUM CHANNEL:\n{missing_perms:?}");
		}
	}

	if guild.id == bot.config.guild_id
		&& !bot
			.discord_cache
			.permissions()
			.root(bot.user_id, bot.config.guild_id)?
			.contains(Permissions::VIEW_AUDIT_LOG)
	{
		warn!("bot is missing VIEW_AUDIT_LOG in guild_id, so it won't see bans/kicks/mutes");
	}

	Ok(())
}
