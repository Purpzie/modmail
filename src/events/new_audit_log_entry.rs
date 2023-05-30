use crate::bot::Bot;
use std::{fmt::Write as _, sync::Arc};
use twilight::model::{
	gateway::payload::incoming::GuildAuditLogEntryCreate,
	guild::audit_log::{AuditLogChange, AuditLogEventType},
};

pub async fn handle(bot: Arc<Bot>, entry: GuildAuditLogEntryCreate) -> anyhow::Result<()> {
	if entry.guild_id != Some(bot.config.guild_id) {
		return Ok(());
	}
	let (Some(user_id), Some(target_id)) = (entry.user_id, entry.target_id) else {
		return Ok(());
	};
	if user_id == bot.user_id {
		return Ok(());
	}

	match entry.action_type {
		AuditLogEventType::MemberBanAdd | AuditLogEventType::MemberKick => {
			let ticket = match bot.db.ticket_by_user(target_id.cast()).await? {
				Some(ticket) if !ticket.is_open => return Ok(()),
				Some(ticket) => ticket,
				None => return Ok(()),
			};

			let mut notif_text = if entry.action_type == AuditLogEventType::MemberBanAdd {
				format!("🔨 User was banned by <@{user_id}>")
			} else {
				format!("👢 User was kicked by <@{user_id}>")
			};
			match &entry.reason {
				Some(reason) => write!(notif_text, " for `{reason}`.")?,
				None => notif_text.push('.'),
			}

			bot.http
				.create_message(ticket.thread_id)
				.content(&notif_text)?
				.await?;
		},

		// mutes
		AuditLogEventType::MemberUpdate => {
			let Some(mute_expires_at) = entry.changes.iter().find_map(|change| {
				if let AuditLogChange::CommunicationDisabledUntil {
					old: None,
					new: Some(timestamp),
				} = change {
					Some(timestamp)
				} else {
					None
				}
			}) else {
				return Ok(());
			};

			let ticket = match bot.db.ticket_by_user(target_id.cast()).await? {
				Some(data) if !data.is_open => return Ok(()),
				None => return Ok(()),
				Some(data) => data,
			};

			let mut notif_text = format!("🔇 User was muted by <@{user_id}>");
			match &entry.reason {
				Some(reason) => write!(notif_text, " for `{reason}`.")?,
				None => notif_text.push('.'),
			}
			write!(
				notif_text,
				"\nThe mute expires <t:{}:R>",
				mute_expires_at.as_secs()
			)?;

			bot.http
				.create_message(ticket.thread_id)
				.content(&notif_text)?
				.await?;
		},

		_ => (),
	}

	Ok(())
}
