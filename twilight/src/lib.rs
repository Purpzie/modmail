pub extern crate twilight_cache_inmemory as cache;
pub extern crate twilight_gateway as gateway;
pub extern crate twilight_http as http;
pub extern crate twilight_model as model;
pub extern crate twilight_util as util;
pub extern crate twilight_validate as validate;

pub mod id {
	pub use super::model::id::{marker::*, Id};
	use paste::paste;

	macro_rules! id_aliases {
		($($name:ident),*$(,)?) => {
			paste! {$(
				pub type [<$name Id>] = Id<[<$name Marker>]>;
			)*}
		};
	}

	// see https://docs.rs/twilight-model/latest/twilight_model/id/marker/index.html#structs
	id_aliases! {
		Application,
		Attachment,
		AuditLogEntry,
		AutoModerationRule,
		Channel,
		Command,
		CommandVersion,
		Emoji,
		Generic,
		Guild,
		Integration,
		Interaction,
		Message,
		OauthSku,
		OauthTeam,
		Role,
		RoleSubscriptionSku,
		ScheduledEventEntity,
		ScheduledEvent,
		Stage,
		StickerBannerAsset,
		Sticker,
		StickerPack,
		StickerPackSku,
		Tag,
		User,
		Webhook,
	}
}
