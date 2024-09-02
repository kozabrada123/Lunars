-- Add migration script here
CREATE TABLE IF NOT EXISTS players (
   id BIGINT UNSIGNED NOT NULL PRIMARY KEY AUTO_INCREMENT,
   name TEXT NOT NULL,
   rating DOUBLE NOT NULL,
   deviation DOUBLE NOT NULL,
   volatility DOUBLE NOT NULL
);
