#![allow(dead_code)]

use twilight::model::{
	channel::message::MessageFlags,
	http::interaction::{InteractionResponse, InteractionResponseData, InteractionResponseType},
};

pub const DEFER: InteractionResponse = InteractionResponse {
	kind: InteractionResponseType::DeferredChannelMessageWithSource,
	data: None,
};

pub const DEFER_EPHEMERAL: InteractionResponse = InteractionResponse {
	kind: InteractionResponseType::DeferredChannelMessageWithSource,
	data: Some(InteractionResponseData {
		flags: Some(MessageFlags::EPHEMERAL),
		allowed_mentions: None,
		attachments: None,
		choices: None,
		components: None,
		content: None,
		custom_id: None,
		embeds: None,
		title: None,
		tts: None,
	}),
};

pub const BLANK_EMBED_COLOR: u32 = 0x2B2D31;
pub const BLURPLE: u32 = 0x5865F2;
pub const GREEN: u32 = 0x57F287;
pub const YELLOW: u32 = 0xFEE75C;
pub const PINK: u32 = 0xEB459E;
pub const RED: u32 = 0xED4245;
