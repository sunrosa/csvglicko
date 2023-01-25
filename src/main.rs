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

    /// Threshold above which ratings are considered provisional.
    #[arg(
        short = 't',
        long = "provisional-threshold",
        help = "Threshold above which ratings are considered provisional.",
        default_value = "110.0"
    )]
    provisional_threshold: Option<f64>,

    /// Filter out provisional ratings.
    #[arg(
        short = 'p',
        long = "filter-provisional",
        help = "Filter out provisional ratings."
    )]
    filter_provisional: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    // Parse command-line arguments
    let args = Args::parse();

    // Generate all ratings from stdin
    let ratings = rate_stdin()?;

    // Sort ratings descending (highest first)
    let mut ratings_sorted: Vec<_> = ratings.into_iter().collect();
    ratings_sorted.sort_by(|a, b| b.1.rating.partial_cmp(&a.1.rating).unwrap());

    // Output loop
    for rating in ratings_sorted {
        // If the maximum deviation option is set, limit all output to below that number
        if args.maximum_deviation.is_some()
            && rating.1.deviation > args.maximum_deviation.unwrap() as f64
        {
            continue;
        }

        // If the minimum deviation option is set, limit all output to above that number
        if args.minimum_deviation.is_some()
            && rating.1.deviation < args.minimum_deviation.unwrap() as f64
        {
            continue;
        }

        // Filter out provisional ratings if the filter_provisional flag is set
        if args.filter_provisional && rating.1.deviation > args.provisional_threshold.expect("The option PROVISIONAL_THRESHOLD has a default value, and therefore you should never see this panic.") {
            continue;
        }

        // Determine whether the provisional mark should be empty or a question mark
        let mut provisional_mark: &str = " ";
        if rating.1.deviation > args.provisional_threshold.expect("The option PROVISIONAL_THRESHOLD has a default value, and therefore you should never see this panic.") {
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
