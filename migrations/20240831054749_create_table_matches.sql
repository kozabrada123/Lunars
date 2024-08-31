-- Add migration script here
CREATE TABLE IF NOT EXISTS matches (
   id BIGINT UNSIGNED NOT NULL PRIMARY KEY AUTO_INCREMENT,
   player_a BIGINT UNSIGNED NOT NULL,
   player_b BIGINT UNSIGNED NOT NULL,

   score_a TINYINT UNSIGNED NOT NULL,
   score_b TINYINT UNSIGNED NOT NULL,

   ping_a SMALLINT UNSIGNED NOT NULL,
   ping_b SMALLINT UNSIGNED NOT NULL,

   rating_a DOUBLE NOT NULL,
   rating_b DOUBLE NOT NULL,

   deviation_a DOUBLE NOT NULL,
   deviation_b DOUBLE NOT NULL,

   volatility_a DOUBLE NOT NULL,
   volatility_b DOUBLE NOT NULL,

   epoch TIMESTAMP NOT NULL,
	FOREIGN KEY(player_a) REFERENCES players(id),
	FOREIGN KEY(player_b) REFERENCES players(id)
);
