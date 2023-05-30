use crate::util::SqliteId;
use sqlx::{
	sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteRow},
	ConnectOptions, FromRow, Row, SqlitePool,
};
use twilight::id::{ChannelId, UserId};

const DATABASE_PATH: &str = "db.sqlite";

pub struct Database {
	pub connection: SqlitePool,
}

#[derive(Clone)]
pub struct Ticket {
	pub user_id: UserId,
	pub dm_channel_id: ChannelId,
	pub thread_id: ChannelId,
	pub is_open: bool,
	pub blocked: bool,
}

impl<'r> FromRow<'r, SqliteRow> for Ticket {
	fn from_row(row: &'r SqliteRow) -> Result<Self, sqlx::Error> {
		let user_id: SqliteId<UserId> = row.try_get("user_id")?;
		let dm_channel_id: SqliteId<ChannelId> = row.try_get("dm_channel_id")?;
		let thread_id: SqliteId<ChannelId> = row.try_get("thread_id")?;
		Ok(Self {
			user_id: *user_id,
			dm_channel_id: *dm_channel_id,
			thread_id: *thread_id,
			is_open: row.try_get("is_open")?,
			blocked: row.try_get("blocked")?,
		})
	}
}

impl Database {
	pub async fn connect() -> anyhow::Result<Self> {
		let mut db_options = SqliteConnectOptions::new()
			.filename(DATABASE_PATH)
			.create_if_missing(true)
			.journal_mode(SqliteJournalMode::Wal);

		db_options.disable_statement_logging();
		let connection = SqlitePoolOptions::new().connect_with(db_options).await?;
		sqlx::migrate!("./migrations").run(&connection).await?;

		Ok(Self { connection })
	}

	pub async fn ticket_by_user(&self, user_id: UserId) -> anyhow::Result<Option<Ticket>> {
		Ok(sqlx::query_as("SELECT * FROM tickets WHERE user_id = ?")
			.bind(SqliteId(user_id))
			.fetch_optional(&self.connection)
			.await?)
	}

	pub async fn ticket_by_dm_channel(
		&self,
		dm_channel_id: ChannelId,
	) -> anyhow::Result<Option<Ticket>> {
		Ok(
			sqlx::query_as("SELECT * FROM tickets WHERE dm_channel_id = ?")
				.bind(SqliteId(dm_channel_id))
				.fetch_optional(&self.connection)
				.await?,
		)
	}

	pub async fn ticket_by_thread(&self, thread_id: ChannelId) -> anyhow::Result<Option<Ticket>> {
		Ok(sqlx::query_as("SELECT * FROM tickets WHERE thread_id = ?")
			.bind(SqliteId(thread_id))
			.fetch_optional(&self.connection)
			.await?)
	}

	pub async fn delete_ticket(&self, thread_id: ChannelId) -> anyhow::Result<()> {
		sqlx::query("DELETE FROM tickets WHERE thread_id = ?")
			.bind(SqliteId(thread_id))
			.execute(&self.connection)
			.await?;

		Ok(())
	}
}
