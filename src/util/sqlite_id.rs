use anyhow::Context;
use sqlx::{
	sqlite::{SqliteArgumentValue, SqliteRow, SqliteTypeInfo, SqliteValueRef},
	Decode, Encode, FromRow, Row, Sqlite, Type,
};
use std::ops::{Deref, DerefMut};
use twilight::id::Id;

/// Sqlite doesn't support storing [`u64`] directly, so this wrapper stores it as an [`i64`].
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct SqliteId<T>(pub T);

impl<T> From<T> for SqliteId<T> {
	fn from(value: T) -> Self {
		Self(value)
	}
}

impl<T> Deref for SqliteId<Id<T>> {
	type Target = Id<T>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl<T> DerefMut for SqliteId<Id<T>> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

impl<T> Type<Sqlite> for SqliteId<Id<T>> {
	fn compatible(ty: &SqliteTypeInfo) -> bool {
		i64::compatible(ty)
	}

	fn type_info() -> SqliteTypeInfo {
		i64::type_info()
	}
}

impl<'q, T> Encode<'q, Sqlite> for SqliteId<Id<T>> {
	fn encode_by_ref(&self, buf: &mut Vec<SqliteArgumentValue>) -> sqlx::encode::IsNull {
		(self.0.get() as i64).encode_by_ref(buf)
	}

	fn encode(self, buf: &mut Vec<SqliteArgumentValue>) -> sqlx::encode::IsNull
	where
		Self: Sized,
	{
		(self.0.get() as i64).encode(buf)
	}

	fn produces(&self) -> Option<SqliteTypeInfo> {
		(self.0.get() as i64).produces()
	}

	fn size_hint(&self) -> usize {
		(self.0.get() as i64).size_hint()
	}
}

impl<'r, T> Decode<'r, Sqlite> for SqliteId<Id<T>> {
	fn decode(value: SqliteValueRef<'r>) -> Result<Self, sqlx::error::BoxDynError> {
		let value = i64::decode(value)? as u64;
		let id = Id::new_checked(value).context("value cannot be 0")?;
		Ok(Self(id))
	}
}

impl<'r, T> FromRow<'r, SqliteRow> for SqliteId<Id<T>> {
	fn from_row(row: &'r SqliteRow) -> Result<Self, sqlx::Error> {
		row.try_get(0)
	}
}
