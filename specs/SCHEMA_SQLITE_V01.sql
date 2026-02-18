PRAGMA journal_mode=WAL;
PRAGMA foreign_keys=ON;

CREATE TABLE IF NOT EXISTS users (
  uid TEXT PRIMARY KEY,
  display_name TEXT NOT NULL,
  status TEXT NOT NULL DEFAULT 'active',
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS user_identities (
  identity_id TEXT PRIMARY KEY,
  uid TEXT NOT NULL,
  channel TEXT NOT NULL,
  channel_user_id TEXT NOT NULL,
  is_verified INTEGER NOT NULL DEFAULT 0,
  confidence REAL NOT NULL DEFAULT 1.0,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  UNIQUE(channel, channel_user_id),
  FOREIGN KEY(uid) REFERENCES users(uid) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS scopes (
  scope_id TEXT PRIMARY KEY,
  scope_type TEXT NOT NULL,
  created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS scope_members (
  scope_id TEXT NOT NULL,
  uid TEXT NOT NULL,
  role TEXT NOT NULL DEFAULT 'member',
  added_at TEXT NOT NULL,
  PRIMARY KEY(scope_id, uid),
  FOREIGN KEY(scope_id) REFERENCES scopes(scope_id) ON DELETE CASCADE,
  FOREIGN KEY(uid) REFERENCES users(uid) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS events (
  event_id TEXT PRIMARY KEY,
  uid TEXT NOT NULL,
  scope_id TEXT NOT NULL,
  event_type TEXT NOT NULL,
  event_ts TEXT NOT NULL,
  payload_json TEXT NOT NULL,
  source_channel TEXT,
  source_message_id TEXT,
  idempotency_key TEXT,
  schema_version TEXT NOT NULL DEFAULT '1',
  created_at TEXT NOT NULL,
  FOREIGN KEY(uid) REFERENCES users(uid) ON DELETE CASCADE,
  FOREIGN KEY(scope_id) REFERENCES scopes(scope_id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_events_uid_ts ON events(uid, event_ts DESC);
CREATE INDEX IF NOT EXISTS idx_events_scope_ts ON events(scope_id, event_ts DESC);
CREATE INDEX IF NOT EXISTS idx_events_type_ts ON events(event_type, event_ts DESC);
CREATE UNIQUE INDEX IF NOT EXISTS idx_events_idempotency ON events(scope_id, uid, idempotency_key) WHERE idempotency_key IS NOT NULL;

CREATE TABLE IF NOT EXISTS state (
  scope_id TEXT NOT NULL,
  uid TEXT NOT NULL,
  state_key TEXT NOT NULL,
  value_json TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  PRIMARY KEY(scope_id, uid, state_key)
);

CREATE TABLE IF NOT EXISTS metrics (
  scope_id TEXT NOT NULL,
  uid TEXT NOT NULL,
  metric_key TEXT NOT NULL,
  metric_value REAL,
  metric_json TEXT,
  updated_at TEXT NOT NULL,
  PRIMARY KEY(scope_id, uid, metric_key)
);

CREATE TABLE IF NOT EXISTS topk (
  scope_id TEXT NOT NULL,
  uid TEXT NOT NULL,
  topic TEXT NOT NULL,
  rank INTEGER NOT NULL,
  item_key TEXT NOT NULL,
  weight REAL NOT NULL,
  updated_at TEXT NOT NULL,
  PRIMARY KEY(scope_id, uid, topic, rank)
);

CREATE TABLE IF NOT EXISTS schema_registry (
  schema_id TEXT PRIMARY KEY,
  version TEXT NOT NULL,
  schema_json TEXT NOT NULL,
  is_active INTEGER NOT NULL DEFAULT 1,
  created_at TEXT NOT NULL
);
