use super::Bot;
use crate::{
	database::Ticket,
	util::{formatting, SqliteId},
};
use anyhow::Context;
use std::fmt::Write as _;
use twilight::{
	id::{Id, UserId},
	model::channel::{
		message::{AllowedMentions, MentionType},
		thread::AutoArchiveDuration,
	},
};

impl Bot {
	pub async fn create_ticket(&self, user_id: UserId) -> anyhow::Result<Ticket> {
		let mut ticket = Ticket {
			user_id,
			thread_id: Id::new(1),
			dm_channel_id: Id::new(1),
			is_open: false,
			blocked: false,
		};

		// do this first, so we don't do anything if it fails (like if the user blocked us)
		ticket.dm_channel_id = self
			.http
			.create_private_channel(user_id)
			.await?
			.model()
			.await?
			.id;

		let username = self
			.discord_cache
			.user(user_id)
			.context("user not found")?
			.name
			.clone(); // don't hold cache reference over await point

		let thread = self
			.http
			.create_forum_thread(self.config.forum_channel_id, &username)
			.auto_archive_duration(AutoArchiveDuration::Week)
			.message()
			.content("Creating thread...")?
			.await?
			.model()
			.await?;

		if let Err(err) = self
			.http
			.delete_message(thread.channel.id, thread.message.id)
			.await
		{
			error!(?err);
		}

		ticket.thread_id = thread.channel.id;

		sqlx::query(indoc! {"
			INSERT INTO tickets (user_id, thread_id, dm_channel_id)
			VALUES (?1, ?2, ?3)
		"})
		.bind(SqliteId(ticket.user_id))
		.bind(SqliteId(ticket.thread_id))
		.bind(SqliteId(ticket.dm_channel_id))
		.execute(&self.db.connection)
		.await?;

		Ok(ticket)
	}

	pub async fn open_ticket(
		&self,
		ticket: &mut Ticket,
		send_open_msg: bool,
	) -> anyhow::Result<()> {
		if ticket.is_open {
			return Ok(());
		}

		if let (true, Some(open_msg)) = (send_open_msg, &self.config.open_message) {
			if let Err(err) = self
				.http
				.create_message(ticket.dm_channel_id)
				.content(open_msg)?
				.await
			{
				error!(?err);
			}
		}

		let mut starter_msg_text = String::new();
		for &role_id in &self.config.ping_roles {
			write!(starter_msg_text, "<@&{role_id}> ")?;
		}
		if !self.config.ping_roles.is_empty() {
			starter_msg_text.push_str("\n\n");
		}
		write!(starter_msg_text, "<@{}>", ticket.user_id)?;

		let mut allow_role_pings = AllowedMentions::default();
		allow_role_pings.parse.push(MentionType::Roles);

		self.http
			.create_message(ticket.thread_id)
			.content(&starter_msg_text)?
			.allowed_mentions(Some(&allow_role_pings))
			.embeds(&[formatting::user_info_embed(self, ticket.user_id).await?])?
			.await?;

		ticket.is_open = true;
		sqlx::query("UPDATE tickets SET is_open = TRUE WHERE user_id = ?")
			.bind(SqliteId(ticket.user_id))
			.execute(&self.db.connection)
			.await?;

		Ok(())
	}

	pub async fn close_ticket(
		&self,
		ticket: &mut Ticket,
		send_close_msg: bool,
	) -> anyhow::Result<()> {
		if !ticket.is_open {
			return Ok(());
		}

		if let (true, Some(close_msg)) = (send_close_msg, &self.config.close_message) {
			if let Err(err) = self
				.http
				.create_message(ticket.dm_channel_id)
				.content(close_msg)?
				.await
			{
				error!(?err);
			}
		}

		ticket.is_open = false;
		sqlx::query("UPDATE tickets SET is_open = FALSE WHERE user_id = ?")
			.bind(SqliteId(ticket.user_id))
			.execute(&self.db.connection)
			.await?;

		if let Err(err) = self
			.http
			.update_thread(ticket.thread_id)
			.archived(true)
			.await
		{
			error!(?err);
		}

		Ok(())
	}
}
