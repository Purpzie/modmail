use twilight::{
	model::http::interaction::{
		InteractionResponse, InteractionResponseData, InteractionResponseType,
	},
	util::builder::InteractionResponseDataBuilder,
};

pub trait InteractionResponseDataExt {
	fn into_response(self) -> InteractionResponse;
}

impl InteractionResponseDataExt for InteractionResponseData {
	fn into_response(self) -> InteractionResponse {
		InteractionResponse {
			kind: InteractionResponseType::ChannelMessageWithSource,
			data: Some(self),
		}
	}
}

impl InteractionResponseDataExt for InteractionResponseDataBuilder {
	fn into_response(self) -> InteractionResponse {
		self.build().into_response()
	}
}
