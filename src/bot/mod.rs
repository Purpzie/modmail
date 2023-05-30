use crate::{config::Config, database::Database, util::Tasks};
use std::{
	sync::atomic::{AtomicBool, Ordering},
	time::Duration,
};
use twilight::{
	cache::{InMemoryCache, ResourceType},
	gateway::{CloseFrame, Intents, MessageSender, Shard, ShardId},
	http::{client::InteractionClient, Client},
	id::{ApplicationId, UserId},
};

mod modmail;

const INTENTS: Intents = {
	use Intents as I;
	I::empty()
		.union(I::DIRECT_MESSAGES)
		.union(I::GUILDS)
		.union(I::GUILD_MEMBERS)
		.union(I::GUILD_MODERATION)
};

const CACHE_TYPES: ResourceType = {
	use ResourceType as R;
	R::empty()
		.union(R::CHANNEL)
		.union(R::GUILD)
		.union(R::MEMBER)
		.union(R::ROLE)
		.union(R::USER)
};

pub struct Bot {
	pub config: Config,
	pub app_id: ApplicationId,
	pub user_id: UserId,
	pub discord_cache: InMemoryCache,
	pub http: Client,
	pub db: Database,
	pub stopping: AtomicBool,
	pub tasks: Tasks,
	pub discord_websocket: MessageSender,
}

impl Bot {
	pub async fn init() -> anyhow::Result<(Self, Shard)> {
		let config = Config::load()?;

		let http = Client::builder()
			.token(config.token.clone())
			.default_allowed_mentions(Default::default()) // no pings
			.build();

		// get the bot's ids
		let app_id = http.current_user_application().await?.model().await?.id;
		let user_id = http.current_user().await?.model().await?.id;

		let discord_cache = InMemoryCache::builder()
			.resource_types(CACHE_TYPES)
			.message_cache_size(0)
			.build();

		let shard = Shard::new(ShardId::ONE, config.token.clone(), INTENTS);
		let db = Database::connect().await?;

		let bot = Bot {
			config,
			http,
			app_id,
			user_id,
			discord_cache,
			db,
			stopping: AtomicBool::new(false),
			tasks: Tasks::new(),
			discord_websocket: shard.sender(),
		};

		// log session information
		let sessions = bot
			.http
			.gateway()
			.authed()
			.await?
			.model()
			.await?
			.session_start_limit;
		debug!(
			reset_after = ?Duration::from_millis(sessions.reset_after),
			"{}/{} sessions remaining",
			sessions.remaining,
			sessions.total,
		);

		// sync commands with discord
		let cmd_info = crate::commands::info();
		bot.interact()
			.set_guild_commands(bot.config.forum_guild_id, &cmd_info)
			.await?;
		if bot.config.guild_id != bot.config.forum_guild_id {
			bot.interact()
				.set_guild_commands(bot.config.guild_id, &cmd_info)
				.await?;
		}

		Ok((bot, shard))
	}

	pub fn stop(&self) {
		warn!("stopping");
		self.stopping.store(true, Ordering::Release);
		if let Err(err) = self.discord_websocket.close(CloseFrame::NORMAL) {
			error!(?err);
		}
	}

	pub async fn finish_shutdown(&self) {
		self.db.connection.close().await;
	}

	pub fn interact(&self) -> InteractionClient {
		self.http.interaction(self.app_id)
	}
}
