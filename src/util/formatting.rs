use super::BLURPLE;
use crate::bot::Bot;
use anyhow::Context;
use twilight::{
	id::{GuildId, UserId},
	model::{
		channel::message::{
			embed::EmbedAuthor,
			sticker::{MessageSticker, StickerFormatType},
			Embed,
		},
		util::ImageHash,
	},
	util::{
		builder::embed::{EmbedAuthorBuilder, EmbedBuilder, EmbedFieldBuilder, ImageSource},
		snowflake::Snowflake,
	},
};

pub async fn user_info_embed(bot: &Bot, user_id: UserId) -> anyhow::Result<Embed> {
	let user = bot.discord_cache.user(user_id).context("user not found")?;
	let member = bot
		.discord_cache
		.member(bot.config.guild_id, user_id)
		.context("member not found")?;

	let mut embed = EmbedBuilder::new()
		.color(BLURPLE)
		.description(format!(
			"**Joined server <t:{0}:R>** on <t:{0}>\n**Registered <t:{1}:R>** on <t:{1}>",
			member.joined_at().as_secs(),
			user_id.timestamp() / 1_000,
		))
		.field(
			EmbedFieldBuilder::new("User ID", format!("{user_id}"))
				.inline()
				.build(),
		);

	if let Some(hash) = user.avatar {
		embed = embed.thumbnail(ImageSource::url(user_avatar_url(user_id, hash))?);
	}

	let mut author = EmbedAuthorBuilder::new(&user.name);
	if let Some(hash) = member.avatar() {
		author = author.icon_url(ImageSource::url(member_avatar_url(
			bot.config.guild_id,
			user_id,
			hash,
		))?);
	}
	embed = embed.author(author.build());

	let mut roles_list = String::new();
	let mut is_first = true;
	for &role_id in member.roles() {
		// skip @everyone
		if role_id.cast() == bot.config.guild_id {
			continue;
		}
		if let Some(role) = bot.discord_cache.role(role_id) {
			if is_first {
				is_first = false;
			} else {
				roles_list.push_str(", ");
			}
			roles_list.push_str(&role.name);
		}
	}

	// don't hold cache references over await point
	drop(user);
	drop(member);

	if let Ok(dm_channel) = bot.http.create_private_channel(user_id).await {
		let dm_channel_id = dm_channel.model().await?.id;
		embed = embed.field(
			EmbedFieldBuilder::new("DM channel ID", format!("{dm_channel_id}"))
				.inline()
				.build(),
		);
	}

	if !roles_list.is_empty() {
		embed = embed.field(EmbedFieldBuilder::new("Roles", roles_list).build());
	}

	Ok(embed.build())
}

pub fn user_avatar_url(user_id: UserId, hash: ImageHash) -> String {
	let mut url = format!("https://cdn.discordapp.com/avatars/{user_id}/{hash}");
	if hash.is_animated() {
		url.push_str(".gif");
	} else {
		url.push_str(".png");
	}
	url
}

pub fn member_avatar_url(guild_id: GuildId, user_id: UserId, hash: ImageHash) -> String {
	let mut url =
		format!("https://cdn.discordapp.com/guilds/{guild_id}/users/{user_id}/avatars/{hash}");
	if hash.is_animated() {
		url.push_str(".gif");
	} else {
		url.push_str(".png");
	}
	url
}

pub fn sticker_url(sticker: &MessageSticker) -> Option<String> {
	if matches!(
		sticker.format_type,
		StickerFormatType::Lottie | StickerFormatType::Unknown(_)
	) {
		return None;
	}

	let mut url = format!("https://cdn.discordapp.com/stickers/{}.", sticker.id);
	url.push_str(match sticker.format_type {
		StickerFormatType::Png => "png",
		StickerFormatType::Apng => "apng",
		StickerFormatType::Gif => "gif",
		_ => return None,
	});

	Some(url)
}

pub fn embed_author(user_id: UserId, name: &str, avatar: Option<ImageHash>) -> EmbedAuthor {
	let mut author = EmbedAuthorBuilder::new(name);
	if let Some(hash) = avatar {
		author =
			author.icon_url(ImageSource::url(user_avatar_url(user_id, hash)).expect("impossible"));
	}
	author.build()
}
