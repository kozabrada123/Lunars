use chrono::{DateTime, Utc};
use log::debug;

use crate::types::entities::player::Player;

use super::DbConnection;

#[derive(Clone, PartialEq, PartialOrd, Debug, Default)]
/// Additional query parameters, such as ?max_rating, ?mix_rating, ?after, ?before, ?has_player
pub struct QueryParameters {
    // Players
    pub max_rating: Option<f64>,
    pub min_rating: Option<f64>,

    pub max_deviation: Option<f64>,
    pub min_deviation: Option<f64>,

    pub max_volatility: Option<f64>,
    pub min_volatility: Option<f64>,

    // Matches
    pub after: Option<DateTime<Utc>>,
    pub before: Option<DateTime<Utc>>,
    pub season: Option<u64>,
    pub has_player: Option<Vec<String>>,

    // Rating periods
    pub start_after: Option<DateTime<Utc>>,
    pub start_before: Option<DateTime<Utc>>,
    pub end_after: Option<DateTime<Utc>>,
    pub end_before: Option<DateTime<Utc>>,

    // General
    pub sort: Option<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

impl QueryParameters {
    /// Filters a list of players with the parameters
    pub fn apply_to_players_vec(&self, vec: Vec<Player>) -> Vec<Player> {
        let mut filtered = vec
            .into_iter()
            .filter(|player| {
                let mut fits = true;

                if let Some(max_rating) = self.max_rating {
                    fits = fits & (player.get_public_rating() <= max_rating);
                }

                if let Some(min_rating) = self.min_rating {
                    fits = fits & (player.get_public_rating() >= min_rating);
                }

                if let Some(max_deviation) = self.max_deviation {
                    fits = fits & (player.get_public_deviation() <= max_deviation);
                }

                if let Some(min_deviation) = self.min_deviation {
                    fits = fits & (player.get_public_deviation() >= min_deviation);
                }

                if let Some(max_volatility) = self.max_volatility {
                    fits = fits & (player.volatility <= max_volatility);
                }

                if let Some(min_volatility) = self.min_volatility {
                    fits = fits & (player.volatility >= min_volatility);
                }

                fits
            })
            .collect::<Vec<Player>>();

        // I am going to get
        // a deep vain thrombosis
        if let Some(sort_options) = &self.sort {
            let sorts = parse_sort_paramter(&sort_options);

            filtered.sort_by(|a, b| {
                let (first_column, first_type) = &sorts[0];

                let order_inverted = first_type.as_str() == "DESC";

                let mut ordering = match first_column.as_str() {
                    "id" => a.id.cmp(&b.id),
                    "rating" => a.rating.partial_cmp(&b.rating).unwrap(),
                    "deviation" => a.deviation.partial_cmp(&b.deviation).unwrap(),
                    "volatility" => a.volatility.partial_cmp(&b.volatility).unwrap(),
                    _ => panic!("Fuck this"),
                };

                if order_inverted {
                    ordering = ordering.reverse()
                }

                for ordering_index in 1..sorts.len() {
                    let (column, order_type) = &sorts[ordering_index];

                    ordering = ordering.then_with(|| {
                        let order_inverted = order_type.as_str() == "DESC";

                        let mut inner_ordering = match column.as_str() {
                            "id" => a.id.cmp(&b.id),
                            "rating" => a.rating.partial_cmp(&b.rating).unwrap(),
                            "deviation" => a.deviation.partial_cmp(&b.deviation).unwrap(),
                            "volatility" => a.volatility.partial_cmp(&b.volatility).unwrap(),
                            _ => panic!("Fuck this"),
                        };

                        if order_inverted {
                            inner_ordering = inner_ordering.reverse()
                        }

                        inner_ordering
                    })
                }

                ordering
            })
        }

        // Limit and offset...
        let mut starting_index = 0;
        let mut ending_index = filtered.len();

        if let Some(offset) = self.offset {
            starting_index += offset;
        }

        if let Some(limit) = self.limit {
            ending_index = starting_index + limit;
        }

        filtered[starting_index..ending_index].to_vec()
    }
}

/// Parses the string sort url parameter.
///
/// Returns an ordered vec of sort options, where the first element is the column and the second is
/// "ASC" or "DESC"
pub fn parse_sort_paramter(sort_options: &str) -> Vec<(String, String)> {
    let mut sorts = Vec::<&str>::new();
    let mut output_sorts = Vec::new();

    if sort_options.contains(",") {
        sorts = sort_options.split(",").collect();
    } else {
        sorts.push(&sort_options);
    }

    for sort in sorts {
        let column: String;
        let mut sort_type = "".to_string();

        if sort.contains("|") {
            let split = sort.split("|").collect::<Vec<&str>>();
            column = split[0].to_string().to_lowercase();

            let temp_type = split[1].to_string().to_lowercase();

            match temp_type.as_str() {
                "asc" => {
                    sort_type = "ASC".to_string();
                }
                "desc" => {
                    sort_type = "DESC".to_string();
                }
                &_ => {
                    // Invalid sort type, we'll set it to a default later
                }
            }
        } else {
            // We can assume (?) that the entire sort type is the column
            column = sort.to_string().to_lowercase();
        }

        // Verify column, we can't escape it
        match column.as_str() {
            "id" | "name" | "rating" | "deviation" | "volatility" | "player_a" | "player_b"
            | "score_a" | "score_b" | "ping_a" | "ping_b" | "rating_a" | "rating_b"
            | "deviation_a" | "deviation_b" | "volatility_a" | "volatility_b" | "epoch" => {}
            _ => {
                log::warn!(
                    "Is this sql injection, or me being dumb? Tried to sort by column {:?}",
                    column
                );
                continue;
            }
        }

        if sort_type == "" {
            // We don't have one, use the default
            match column.as_str() {
                "id" => {
                    sort_type = "ASC".to_string();
                }
                "player_a" => {
                    sort_type = "ASC".to_string();
                }
                "player_b" => {
                    sort_type = "ASC".to_string();
                }
                "rating" | "rating_a" | "rating_b" => {
                    sort_type = "DESC".to_string();
                }
                "deviation" | "deviation_a" | "deviation_b" => {
                    sort_type = "ASC".to_string();
                }
                "volatility" | "volatility_a" | "volatility_b" => {
                    sort_type = "ASC".to_string();
                }
                // Show latest games first
                "epoch" => {
                    sort_type = "DESC".to_string();
                }
                &_ => {
                    // None of the specific ones, default to ascending
                    sort_type = "ASC".to_string();
                }
            }
        }

        output_sorts.push((column, sort_type));
    }

    output_sorts
}

impl DbConnection {
    /// Function that takes url args (?max, ?min, ?player...) and makes an sql query
    ///
    /// Starting parameter index is the next index to use. By default, this should be 1.
    /// Usually you will have sql which will already use parameters, if you do, then pass
    /// the last index + 1 as a starting index.
    /*

    Usage:
    base = "SELECT * FROM players"
    request is your request

    based on determined parameters, this returns the modified query.

    for example, if request here has a parameter ?max=x, then this will return
    something like "SELECT * FROM players WHERE rank <= x"

    valid params to be implemented:

    /api/players:
    ?max=x, where x is the max rank value
    ?min=x, where x is the minimum rank value

    /api/matches:
    ?after=x, where x is a unix timestamp
    ?before=x, where x is a unix timestamp
    ?has_player=x, where x is a player id or name that we want to be in the match

    */
    pub async fn add_to_query(
        &mut self,
        base: &'static str,
        parameters: QueryParameters,
    ) -> (String, Vec<String>) {
        let mut query = base.to_string();

        // Added parameters we'll need to bind
        let mut added_parameters = Vec::new();

        let mut first_parameter = true;

        if let Some(max_rating) = parameters.max_rating {
            debug!("Got valid url parameter max_rating: {}", max_rating);

            let mut to_add = String::new();

            match first_parameter {
                true => {
                    to_add.push_str(" WHERE ");
                    first_parameter = false;
                }
                false => {
                    to_add.push_str(" AND ");
                }
            }

            to_add.push_str("rating <= ?");
            query.push_str(to_add.as_str());

            added_parameters.push(max_rating.to_string());
        }

        if let Some(min_rating) = parameters.min_rating {
            debug!("Got valid url parameter min_rating: {}", min_rating);

            let mut to_add = String::new();

            match first_parameter {
                true => {
                    to_add.push_str(" WHERE ");
                    first_parameter = false;
                }
                false => {
                    to_add.push_str(" AND ");
                }
            }

            to_add.push_str("rating >= ?");
            query.push_str(to_add.as_str());

            added_parameters.push(min_rating.to_string());
        }

        if let Some(max_deviation) = parameters.max_deviation {
            debug!("Got valid url parameter max_deviation: {}", max_deviation);

            let mut to_add = String::new();

            match first_parameter {
                true => {
                    to_add.push_str(" WHERE ");
                    first_parameter = false;
                }
                false => {
                    to_add.push_str(" AND ");
                }
            }

            to_add.push_str("deviation <= ?");
            query.push_str(to_add.as_str());

            added_parameters.push(max_deviation.to_string());
        }

        if let Some(min_deviation) = parameters.min_deviation {
            debug!("Got valid url parameter min_deviation: {}", min_deviation);

            let mut to_add = String::new();

            match first_parameter {
                true => {
                    to_add.push_str(" WHERE ");
                    first_parameter = false;
                }
                false => {
                    to_add.push_str(" AND ");
                }
            }

            to_add.push_str("deviation >= ?");
            query.push_str(to_add.as_str());

            added_parameters.push(min_deviation.to_string());
        }

        if let Some(max_volatility) = parameters.max_volatility {
            debug!("Got valid url parameter max_volatility: {}", max_volatility);

            let mut to_add = String::new();

            match first_parameter {
                true => {
                    to_add.push_str(" WHERE ");
                    first_parameter = false;
                }
                false => {
                    to_add.push_str(" AND ");
                }
            }

            to_add.push_str("volatility <= ?");
            query.push_str(to_add.as_str());

            added_parameters.push(max_volatility.to_string());
        }

        if let Some(min_volatility) = parameters.min_volatility {
            debug!("Got valid url parameter min_volatility: {}", min_volatility);

            let mut to_add = String::new();

            match first_parameter {
                true => {
                    to_add.push_str(" WHERE ");
                    first_parameter = false;
                }
                false => {
                    to_add.push_str(" AND ");
                }
            }

            to_add.push_str("volatility >= ?");
            query.push_str(to_add.as_str());

            added_parameters.push(min_volatility.to_string());
        }

        if let Some(has_player_requirements) = parameters.has_player {
            debug!(
                "Got valid url parameter has_player: {:?}",
                has_player_requirements
            );

            for player_query in has_player_requirements {
                let player_res = self.get_player_by_id_or_name(&player_query).await;

                if player_res.is_none() {
                    continue;
                }

                let player = player_res.unwrap();

                let mut to_add = String::new();

                match first_parameter {
                    true => {
                        to_add.push_str(" WHERE ");
                        first_parameter = false;
                    }
                    false => {
                        to_add.push_str(" AND ");
                    }
                }

                to_add.push_str("(player_a = ? OR player_b = ?)");
                query.push_str(to_add.as_str());

                added_parameters.push(player.id.to_string());
                added_parameters.push(player.id.to_string());
            }
        }

        if let Some(before) = parameters.before {
            debug!("Got valid url parameter before: {}", before);

            let mut to_add = String::new();

            match first_parameter {
                true => {
                    to_add.push_str(" WHERE ");
                    first_parameter = false;
                }
                false => {
                    to_add.push_str(" AND ");
                }
            }

            to_add.push_str("epoch < ?");
            query.push_str(to_add.as_str());

            added_parameters.push(before.to_string());
        }

        if let Some(after) = parameters.after {
            debug!("Got valid url parameter after: {}", after);

            let mut to_add = String::new();

            match first_parameter {
                true => {
                    to_add.push_str(" WHERE ");
                    first_parameter = false;
                }
                false => {
                    to_add.push_str(" AND ");
                }
            }

            to_add.push_str("epoch > ?");
            query.push_str(to_add.as_str());

            added_parameters.push(after.to_string());
        }

        if let Some(season) = parameters.season {
            debug!("Got valid url parameter season: {}", season);

            let mut to_add = String::new();

            match first_parameter {
                true => {
                    to_add.push_str(" WHERE ");
                    first_parameter = false;
                }
                false => {
                    to_add.push_str(" AND ");
                }
            }

            to_add.push_str("rating_period = ?");
            query.push_str(to_add.as_str());

            added_parameters.push(season.to_string());
        }

        if let Some(start_before) = parameters.start_before {
            debug!("Got valid url parameter start_before: {}", start_before);

            let mut to_add = String::new();

            match first_parameter {
                true => {
                    to_add.push_str(" WHERE ");
                    first_parameter = false;
                }
                false => {
                    to_add.push_str(" AND ");
                }
            }

            to_add.push_str("start < ?");
            query.push_str(to_add.as_str());

            added_parameters.push(start_before.to_string());
        }

        if let Some(start_after) = parameters.start_after {
            debug!("Got valid url parameter start_after: {}", start_after);

            let mut to_add = String::new();

            match first_parameter {
                true => {
                    to_add.push_str(" WHERE ");
                    first_parameter = false;
                }
                false => {
                    to_add.push_str(" AND ");
                }
            }

            to_add.push_str("start > ?");
            query.push_str(to_add.as_str());

            added_parameters.push(start_after.to_string());
        }

        if let Some(end_before) = parameters.end_before {
            debug!("Got valid url parameter end_before: {}", end_before);

            let mut to_add = String::new();

            match first_parameter {
                true => {
                    to_add.push_str(" WHERE ");
                    first_parameter = false;
                }
                false => {
                    to_add.push_str(" AND ");
                }
            }

            to_add.push_str("end < ?");
            query.push_str(to_add.as_str());

            added_parameters.push(end_before.to_string());
        }

        if let Some(end_after) = parameters.end_after {
            debug!("Got valid url parameter end_after: {}", end_after);

            let mut to_add = String::new();

            match first_parameter {
                true => {
                    to_add.push_str(" WHERE ");
                    first_parameter = false;
                }
                false => {
                    to_add.push_str(" AND ");
                }
            }

            to_add.push_str("end > ?");
            query.push_str(to_add.as_str());

            added_parameters.push(end_after.to_string());
        }

        if let Some(sort_options) = parameters.sort {
            debug!("Got valid url parameter sort: {}", sort_options);

            let mut to_add = String::new();

            // Parse the sort param
            // What we want is something like sort="rank|asc,name|desc"
            // , defines different sorts
            // | is the deliminator between column names and asc / desc

            let sorts = parse_sort_paramter(&sort_options);

            let mut first_sort = true;

            for sort in sorts {
                match first_sort {
                    true => {
                        to_add.push_str(" ORDER BY ");
                        first_sort = false;
                    }
                    false => {
                        to_add.push_str(", ");
                    }
                }

                let (column, sort_type) = sort;

                debug!("Adding to sort {} {}", column, sort_type);

                to_add.push_str(format!("{} {}", column, sort_type).as_str());
            }

            query.push_str(to_add.as_str());
        }

        if let Some(limit) = parameters.limit {
            debug!("Got valid url parameter limit: {}", limit);

            let mut to_add = String::new();

            to_add.push_str(" LIMIT ?");
            query.push_str(to_add.as_str());

            added_parameters.push(limit.to_string());
        }

        if let Some(offset) = parameters.offset {
            debug!("Got valid url parameter offset: {}", offset);

            let mut to_add = String::new();

            to_add.push_str(" OFFSET ?");
            query.push_str(to_add.as_str());

            added_parameters.push(offset.to_string());
        }

        query.push_str(";");

        debug!(
            "Parsed url params into query: {} + {:?} ({})",
            query,
            added_parameters,
            added_parameters.len()
        );

        (query, added_parameters)
    }
}
