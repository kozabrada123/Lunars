-- Add migration script here
CREATE TABLE IF NOT EXISTS recent_requests (
   epoch TIMESTAMP NOT NULL,
	ip TEXT,
   api_key_hash TEXT
);
