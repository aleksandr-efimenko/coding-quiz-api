-- Drop consumer-facing history
DROP TABLE IF EXISTS user_answers;

-- Rename users to developers
ALTER TABLE users RENAME TO developers;

-- Create API Keys table
CREATE TABLE api_keys (
    id UUID PRIMARY KEY,
    developer_id UUID NOT NULL REFERENCES developers(id),
    key_hash VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create Usage Logs table
CREATE TABLE usage_logs (
    id UUID PRIMARY KEY,
    api_key_id UUID NOT NULL REFERENCES api_keys(id),
    endpoint VARCHAR(255) NOT NULL,
    status_code INTEGER NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
