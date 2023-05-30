CREATE TABLE IF NOT EXISTS tickets (
	user_id INTEGER PRIMARY KEY,
	dm_channel_id INTEGER NOT NULL,
	thread_id INTEGER NOT NULL,
	is_open BOOLEAN DEFAULT 0 CHECK(is_open IN (0, 1)),
	blocked BOOLEAN DEFAULT 0 CHECK(blocked IN (0, 1))
);

CREATE UNIQUE INDEX IF NOT EXISTS ticket_dm_channels ON tickets (dm_channel_id);
CREATE UNIQUE INDEX IF NOT EXISTS ticket_threads ON tickets (thread_id);

CREATE TABLE IF NOT EXISTS messages (
	user_id INTEGER NOT NULL,
	dm_msg_id INTEGER NOT NULL,
	thread_msg_id INTEGER NOT NULL,
	thread_update_msg_id INTEGER DEFAULT NULL,
	FOREIGN KEY (user_id) REFERENCES tickets (user_id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS message_users ON messages (user_id);
