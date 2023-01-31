use clap::Parser;
use colored::Colorize;
use std::error::Error;

mod local_glicko2;

/// Command-line arguments for csvglicko.
#[derive(Parser)]
#[command(about)]
struct Args {
    /// CSV file path to calculate ratings for.
    #[arg(help = "CSV file path to calculate ratings for.")]
    csv: String,

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
    provisional_threshold: f64,

    /// Default rating to be used for players.
    #[arg(
        short = 'r',
        long = "default-rating",
        help = "Default rating to be used for players.",
        default_value = "1500.0"
    )]
    default_rating: f64,

    /// Default rating deviation to be used for players.
    #[arg(
        long = "default-deviation",
        help = "Default rating deviation to be used for players.",
        default_value = "350.0"
    )]
    default_rating_deviation: f64,

    /// Default volatility to be used for players.
    #[arg(
        long = "default-volatility",
        help = "Default volatility to be used for players.",
        default_value = "0.06"
    )]
    default_volatility: f64,

    /// Tau value used in the Glicko-2 configuration.
    #[arg(
        long = "default-tau",
        help = "Tau value used in the Glicko-2 configuration.",
        default_value = "0.5"
    )]
    default_tau: f64,

    /// Convergence tolerance to be used in the Glicko-2 configuration.
    #[arg(
        long = "default-tolerance",
        help = "Convergence tolerance to be used in the Glicko-2 configuration.",
        default_value = "0.000001"
    )]
    default_convergence_tolerance: f64,

    /// Filter out provisional ratings.
    #[arg(
        short = 'p',
        long = "filter-provisional",
        help = "Filter out provisional ratings."
    )]
    filter_provisional: bool,

    /// Sort ascending by rating deviation.
    #[arg(
        short = 'e',
        long = "sort-deviation",
        help = "Sort ascending by rating deviation."
    )]
    sort_rating_deviation: bool,

    /// Sort descending by volatility.
    #[arg(
        short = 'v',
        long = "sort-volatility",
        help = "Sort descending by volatility."
    )]
    sort_volatility: bool,

    /// Reverse sorting.
    #[arg(short = 's', long = "sort-reverse", help = "Reverse sorting.")]
    sort_reverse: bool,

    /// Output result limit.
    #[arg(short = 'l', long = "result-limit", help = "Output result limit.")]
    result_limit: Option<u32>,

    /// Disable invisible indexes
    #[arg(
        short = 'i',
        long = "invisible-indexes",
        help = "Disable invisible indexes when filtering. (i.e. No gaps in printed index)"
    )]
    invisible_indexes: bool,
}

/// A representation of one rated player.
pub struct Player {
    /// The player's Glicko-2 rating.
    pub rating: skillratings::glicko2::Glicko2Rating,
    /// The change (up or down) induced by the player's last game.
    pub latest_change: f64,
}

fn main() {
    // Parse command-line arguments
    let args = Args::parse();

    // Initialize Glicko-2 rating and default config
    let glicko2_config = skillratings::glicko2::Glicko2Config {
        tau: args.default_tau,
        convergence_tolerance: args.default_convergence_tolerance,
        ..Default::default()
    };
    let glicko2_default_rating = skillratings::glicko2::Glicko2Rating {
        rating: args.default_rating,
        deviation: args.default_rating_deviation,
        volatility: args.default_volatility,
        ..Default::default()
    };

    // Generate all ratings from stdin
    let ratings = match rate_file(&glicko2_config, &glicko2_default_rating, &args.csv) {
        Ok(ratings) => ratings,
        Err(e) => {
            println!(
                "There was a problem opening or reading the file \"{}\": {}",
                args.csv, e
            );
            return;
        }
    };
    let mut ratings_sorted: Vec<_> = ratings.into_iter().collect();

    // Sort ratings according to options.
    if args.sort_rating_deviation {
        if !args.sort_reverse {
            ratings_sorted.sort_by(|a, b| {
                a.1.rating
                    .deviation
                    .partial_cmp(&b.1.rating.deviation)
                    .unwrap()
            });
        } else {
            ratings_sorted.sort_by(|a, b| {
                b.1.rating
                    .deviation
                    .partial_cmp(&a.1.rating.deviation)
                    .unwrap()
            });
        }
    } else if args.sort_volatility {
        if !args.sort_reverse {
            ratings_sorted.sort_by(|a, b| {
                b.1.rating
                    .volatility
                    .partial_cmp(&a.1.rating.volatility)
                    .unwrap()
            });
        } else {
            ratings_sorted.sort_by(|a, b| {
                a.1.rating
                    .volatility
                    .partial_cmp(&b.1.rating.volatility)
                    .unwrap()
            });
        }
    } else {
        if !args.sort_reverse {
            ratings_sorted
                .sort_by(|a, b| b.1.rating.rating.partial_cmp(&a.1.rating.rating).unwrap());
        } else {
            ratings_sorted
                .sort_by(|a, b| a.1.rating.rating.partial_cmp(&b.1.rating.rating).unwrap());
        }
    }

    // When removing an index through filtering, add a number to index_subtraction to not include invisible indexes
    let mut index_subtraction: i32 = 0;

    // Output loop
    for (index, player) in ratings_sorted.iter().enumerate() {
        if args.result_limit.is_some() && index >= args.result_limit.unwrap() as usize {
            break;
        }

        // If the maximum deviation option is set, limit all output to below that number
        if args.maximum_deviation.is_some()
            && player.1.rating.deviation > args.maximum_deviation.unwrap() as f64
        {
            if args.invisible_indexes {
                index_subtraction += 1;
            }

            continue;
        }

        // If the minimum deviation option is set, limit all output to above that number
        if args.minimum_deviation.is_some()
            && player.1.rating.deviation < args.minimum_deviation.unwrap() as f64
        {
            if args.invisible_indexes {
                index_subtraction += 1;
            }
            continue;
        }

        // Filter out provisional ratings if the filter_provisional flag is set
        if args.filter_provisional && player.1.rating.deviation > args.provisional_threshold {
            if args.invisible_indexes {
                index_subtraction += 1;
            }
            continue;
        }

        // Determine whether the provisional mark should be empty or a question mark
        let mut provisional_mark: &str = " ";
        if player.1.rating.deviation > args.provisional_threshold {
            provisional_mark = "?";
        }

        println!(
            "{:0index_width$}. {}{} ({}) {} {} {}",
            (index as i32) + 1 - index_subtraction,
            format!("{:07.2}", player.1.rating.rating).red(),
            provisional_mark.yellow(),
            format!("{:+07.2}", player.1.latest_change),
            format!("{:03.0}", player.1.rating.deviation).cyan(),
            format!("{:.8}", player.1.rating.volatility).purple(),
            player.0.to_string().blue(),
            index_width = ratings_sorted.len().to_string().len()
        );
    }
}

/// Generate ratings for all players in the csv file passed in through stdin.
///
/// # Arguments
///
/// * `glicko2_config` - The Glicko-2 configuration to be used in rating calculation.
/// * `glicko2_default_rating` - The default Glicko-2 rating to be used for newly-instantiated players.
fn rate_file(
    glicko2_config: &skillratings::glicko2::Glicko2Config,
    glicko2_default_rating: &skillratings::glicko2::Glicko2Rating,
    file_path: &String,
) -> Result<std::collections::HashMap<String, Player>, Box<dyn Error>> {
    let mut players: std::collections::HashMap<String, Player> = std::collections::HashMap::new();

    let file = std::fs::File::open(file_path)?;

    let mut reader = csv::Reader::from_reader(file);
    for result in reader.records() {
        // Unwrap the line
        let record = result?;

        // Get the player names from the csv line
        let player_1_name = record.get(0).unwrap().to_string();
        let player_2_name = record.get(1).unwrap().to_string();

        // Skip game if a player is fighting themselves somehow
        if player_1_name == player_2_name {
            continue;
        }

        // Get the outcome of the game from the csv line
        let outcome: f64 = record.get(2).unwrap().parse().unwrap();

        // Get players from storage, or create them otherwise
        let mut player_1_rating: skillratings::glicko2::Glicko2Rating =
            glicko2_default_rating.clone();
        if players.contains_key(&player_1_name) {
            player_1_rating = players.get(&player_1_name).unwrap().rating;
        }
        let mut player_2_rating: skillratings::glicko2::Glicko2Rating =
            glicko2_default_rating.clone();
        if players.contains_key(&player_2_name) {
            player_2_rating = players.get(&player_2_name).unwrap().rating;
        }

        // Rate the game
        let (new_player_1_rating, new_player_2_rating) = local_glicko2::glicko2(
            &player_1_rating,
            &player_2_rating,
            &outcome,
            &glicko2_config,
        );

        let player_1_rating_change = new_player_1_rating.rating - player_1_rating.rating;
        let player_2_rating_change = new_player_2_rating.rating - player_2_rating.rating;

        // Save player ratings to player_ratings
        players.insert(
            player_1_name,
            Player {
                rating: new_player_1_rating,
                latest_change: player_1_rating_change,
            },
        );
        players.insert(
            player_2_name,
            Player {
                rating: new_player_2_rating,
                latest_change: player_2_rating_change,
            },
        );
    }

    Ok(players)
}
