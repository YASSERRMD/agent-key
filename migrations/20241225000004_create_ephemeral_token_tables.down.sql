-- Rollback ephemeral token tables
DROP INDEX IF EXISTS idx_sdk_sessions_agent_id;
DROP INDEX IF EXISTS idx_token_usage_log_timestamp;
DROP INDEX IF EXISTS idx_token_usage_log_agent_id;
DROP INDEX IF EXISTS idx_token_usage_log_jti;
DROP INDEX IF EXISTS idx_ephemeral_tokens_expires_at;
DROP INDEX IF EXISTS idx_ephemeral_tokens_status;
DROP INDEX IF EXISTS idx_ephemeral_tokens_credential_id;
DROP INDEX IF EXISTS idx_ephemeral_tokens_agent_id;

DROP TABLE IF EXISTS sdk_sessions;
DROP TABLE IF EXISTS token_usage_log;
DROP TABLE IF EXISTS ephemeral_tokens;
