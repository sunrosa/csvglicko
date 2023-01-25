use clap::Parser;
use colored::Colorize;
use std::error::Error;

mod local_glicko2;

/// Command-line arguments for csvglicko.
#[derive(Parser)]
#[command(about)]
struct Args {
    /// Maximum rating deviation to filter output with.
    #[arg(
        short = 'd',
        long = "maximum-deviation",
        help = "Maximum rating deviation to filter output with."
    )]
    maximum_deviation: Option<u32>,

    /// Minimum rating deviation to filter output with.
    #[arg(
        long = "minimum-deviation",
        help = "Minimum rating deviation to filter output with."
    )]
    minimum_deviation: Option<u32>,
}

fn main() -> Result<(), Box<dyn Error>> {
    // Parse command-line arguments
    let cli = Args::parse();

    // Generate all ratings from stdin
    let ratings = rate_stdin()?;

    for rating in ratings {
        // If the maximum deviation option is set, limit all output to below that number
        if cli.maximum_deviation.is_some()
            && rating.1.deviation > cli.maximum_deviation.unwrap() as f64
        {
            continue;
        }

        // If the minimum deviation option is set, limit all output to above that number
        if cli.minimum_deviation.is_some()
            && rating.1.deviation < cli.minimum_deviation.unwrap() as f64
        {
            continue;
        }

        let mut provisional_mark: &str = " ";
        if rating.1.deviation > 110.0 {
            provisional_mark = "?";
        }

        println!(
            "{}{} {} {}",
            format!("{:.2}", rating.1.rating).red(),
            provisional_mark.yellow(),
            format!("{:.0}", rating.1.deviation).cyan(),
            rating.0.to_string().blue()
        );
    }

    Ok(())
}

fn rate_stdin(
) -> Result<std::collections::HashMap<String, skillratings::glicko2::Glicko2Rating>, Box<dyn Error>>
{
    let glicko2_config = skillratings::glicko2::Glicko2Config::new();
    let glicko2_default_rating = skillratings::glicko2::Glicko2Rating::new();

    let mut player_ratings: std::collections::HashMap<
        String,
        skillratings::glicko2::Glicko2Rating,
    > = std::collections::HashMap::new();

    let mut reader = csv::Reader::from_reader(std::io::stdin());
    for result in reader.records() {
        // Unwrap the line
        let record = result?;

        // Get the player names from the csv line
        let player_1_name = record.get(0).unwrap().to_string();
        let player_2_name = record.get(1).unwrap().to_string();

        // Get the outcome of the game from the csv line
        let outcome: f64 = record.get(2).unwrap().parse().unwrap();

        // Get players from storage, or create them otherwise
        let mut player_1: skillratings::glicko2::Glicko2Rating = glicko2_default_rating;
        if player_ratings.contains_key(&player_1_name) {
            player_1 = *player_ratings.get(&player_1_name).unwrap();
        }
        let mut player_2: skillratings::glicko2::Glicko2Rating = glicko2_default_rating;
        if player_ratings.contains_key(&player_2_name) {
            player_2 = *player_ratings.get(&player_2_name).unwrap();
        }

        // Rate the game
        let (new_player_1, new_player_2) =
            local_glicko2::glicko2(&player_1, &player_2, &outcome, &glicko2_config);

        // Save player ratings to player_ratings
        player_ratings.insert(player_1_name, new_player_1);
        player_ratings.insert(player_2_name, new_player_2);
    }

    Ok(player_ratings)
}
