-- Add migration script here
CREATE TABLE IF NOT EXISTS players (
   id BIGINT UNSIGNED PRIMARY KEY AUTOINCREMENT,
   name TEXT NOT NULL,
   rating DOUBLE NOT NULL,
   deviation DOUBLE NOT NULL,
   volatility DOUBLE NOT NULL
);
