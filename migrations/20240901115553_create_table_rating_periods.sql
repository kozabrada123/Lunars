-- Add migration script here
CREATE TABLE IF NOT EXISTS rating_periods (
   id BIGINT UNSIGNED NOT NULL PRIMARY KEY AUTO_INCREMENT,
   start TIMESTAMP NOT NULL,
	end TIMESTAMP NOT NULL
);
