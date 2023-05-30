use std::{fs::File, io::BufReader};
use twilight::{
	id::{ChannelId, GuildId, RoleId},
	validate::message::MESSAGE_CONTENT_LENGTH_MAX,
};

const CONFIG_PATH: &str = "config.yml";

#[derive(serde::Deserialize)]
struct RawConfig {
	token: String,
	guild_id: GuildId,
	forum_channel_id: ChannelId,
	#[serde(default)]
	forum_guild_id: Option<GuildId>,
	#[serde(default)]
	ping_roles: Vec<RoleId>,
	#[serde(default)]
	open_message: Option<String>,
	#[serde(default)]
	close_message: Option<String>,
}

pub struct Config {
	pub token: String,
	pub guild_id: GuildId,
	pub forum_channel_id: ChannelId,
	pub forum_guild_id: GuildId,
	pub ping_roles: Vec<RoleId>,
	pub open_message: Option<String>,
	pub close_message: Option<String>,
}

impl Config {
	pub fn load() -> anyhow::Result<Self> {
		let file = File::open(CONFIG_PATH)?;
		let raw_config: RawConfig = serde_yaml::from_reader(BufReader::new(file))?;
		let config = Self {
			token: raw_config.token,
			guild_id: raw_config.guild_id,
			forum_channel_id: raw_config.forum_channel_id,
			forum_guild_id: raw_config.forum_guild_id.unwrap_or(raw_config.guild_id),
			ping_roles: raw_config.ping_roles,
			open_message: raw_config.open_message,
			close_message: raw_config.close_message,
		};

		if config
			.open_message
			.as_ref()
			.map(|s| s.is_empty() || s.len() > MESSAGE_CONTENT_LENGTH_MAX)
			.unwrap_or(false)
		{
			bail!("open_message must be 1-{MESSAGE_CONTENT_LENGTH_MAX} characters in length");
		}

		if config
			.close_message
			.as_ref()
			.map(|s| s.is_empty() || s.len() > MESSAGE_CONTENT_LENGTH_MAX)
			.unwrap_or(false)
		{
			bail!("close_message must be 1-{MESSAGE_CONTENT_LENGTH_MAX} characters in length");
		}

		Ok(config)
	}
}
