use std::error::Error;

use skillratings::glicko2::Glicko2Rating;

fn main() -> Result<(), Box<dyn Error>> {
    let glicko2_config = skillratings::glicko2::Glicko2Config::new();
    let glicko2_default_rating = skillratings::glicko2::Glicko2Rating::new();

    let mut player_ratings: std::collections::HashMap<
        std::string::String,
        skillratings::glicko2::Glicko2Rating,
    > = std::collections::HashMap::new();

    let mut reader = csv::Reader::from_reader(std::io::stdin());
    for result in reader.records() {
        // Unwrap the line
        let record = result?;

        // Set the default outcome to win
        let mut outcome = skillratings::Outcomes::WIN;

        // Get the player names
        let player_1_name = record.get(0).unwrap().to_string();
        let player_2_name = record.get(1).unwrap().to_string();

        // Get the players from storage, or otherwise create new player ratings from the default specified above
        // This can be optimized. It should not insert into the hashmap as default.
        let mut player_1: skillratings::glicko2::Glicko2Rating = glicko2_default_rating;
        {
            player_1 = *player_ratings
                .entry(player_1_name.clone())
                .or_insert(glicko2_default_rating);
        }
        let mut player_2: skillratings::glicko2::Glicko2Rating = glicko2_default_rating;
        {
            player_2 = *player_ratings
                .entry(player_2_name.clone())
                .or_insert(glicko2_default_rating);
        }

        // Set the outcome to a draw if specified in the csv
        if record.get(2).unwrap().to_string() == "1" {
            outcome = skillratings::Outcomes::DRAW;
        }

        // Rate the game
        let (new_player_1, new_player_2) =
            skillratings::glicko2::glicko2(&player_1, &player_2, &outcome, &glicko2_config);

        // Save player ratings to player_ratings
        player_ratings.insert(player_1_name, new_player_1);
        player_ratings.insert(player_2_name, new_player_2);
    }

    for rating in player_ratings {
        println!("{}: {:?}", rating.0, rating.1);
    }
    Ok(())
}
