#![deny(clippy::undocumented_unsafe_blocks)]
#![allow(clippy::tabs_in_doc_comments)]

#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate indoc;
#[macro_use]
extern crate tracing;

use std::{
	process::ExitCode,
	sync::{atomic::Ordering, Arc},
};
use twilight::gateway::{error::ReceiveMessageErrorType, Event};

mod bot;
mod commands;
mod config;
mod database;
mod events;
mod logging;
mod util;

use crate::bot::Bot;

fn main() -> ExitCode {
	logging::init();

	let result = tokio::runtime::Builder::new_multi_thread()
		.enable_all()
		.build()
		.expect("failed to build tokio runtime")
		.block_on(run());

	match result {
		Ok(_) => ExitCode::SUCCESS,
		Err(err) => {
			error!("{err:?}");
			ExitCode::FAILURE
		},
	}
}

async fn run() -> anyhow::Result<()> {
	let (bot, mut shard) = Bot::init().await?;
	let bot = Arc::new(bot);

	// handle shutdown gracefully
	let bot_handle = Arc::clone(&bot);
	let ctrl_c_handler = tokio::spawn(async move {
		match tokio::signal::ctrl_c().await {
			Ok(_) => {
				warn!("ctrl-c received");
				bot_handle.stop();
			},
			Err(err) => error!(?err, "unable to listen for ctrl-c"),
		}
	});

	// event loop
	loop {
		match shard.next_event().await {
			// handle intentional stopping
			Ok(Event::GatewayClose(_)) if bot.stopping.load(Ordering::Acquire) => break,
			Err(err)
				if matches!(err.kind(), ReceiveMessageErrorType::Io)
					&& bot.stopping.load(Ordering::Acquire) =>
			{
				break
			},

			Ok(event) => {
				let bot_handle = Arc::clone(&bot);
				bot.tasks.spawn(async move {
					let kind = event.kind();
					if let Err(err) = events::handle_event(bot_handle, event).await {
						error!(?err, "error handling event ({:?})", kind);
					}
				});
			},

			Err(err) => {
				error!(?err, "error receiving event");
				if err.is_fatal() {
					break;
				}
			},
		}
	}

	bot.tasks.finished().await;
	ctrl_c_handler.abort();
	bot.finish_shutdown().await;

	Ok(())
}
