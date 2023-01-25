// BEGIN MODIFIED CODE FROM https://crates.io/crates/skillratings

pub fn glicko2(
    player_one: &skillratings::glicko2::Glicko2Rating,
    player_two: &skillratings::glicko2::Glicko2Rating,
    outcome: &f64,
    config: &skillratings::glicko2::Glicko2Config,
) -> (
    skillratings::glicko2::Glicko2Rating,
    skillratings::glicko2::Glicko2Rating,
) {
    // First we need to convert the ratings into the glicko-2 scale.
    let player_one_rating = (player_one.rating - 1500.0) / 173.7178;
    let player_two_rating = (player_two.rating - 1500.0) / 173.7178;

    // Same with the deviation.
    let player_one_deviation = player_one.deviation / 173.7178;
    let player_two_deviation = player_two.deviation / 173.7178;

    let outcome1 = outcome.clone();
    let outcome2 = 1.0 - outcome1;

    // We always need the deviation of the opponent in the g function.
    let g1 = g_value(player_two_deviation);
    let g2 = g_value(player_one_deviation);

    let e1 = e_value(player_one_rating, player_two_rating, g1);
    let e2 = e_value(player_two_rating, player_one_rating, g2);

    let v1 = v_value(g1, e1);
    let v2 = v_value(g2, e2);

    let player_one_new_volatility = new_volatility(
        player_one.volatility,
        delta_value(outcome1, v1, g1, e1).powi(2),
        player_one_deviation.powi(2),
        v1,
        config.tau,
        config.convergence_tolerance,
    );
    let player_two_new_volatility = new_volatility(
        player_two.volatility,
        delta_value(outcome2, v2, g2, e2).powi(2),
        player_two_deviation.powi(2),
        v2,
        config.tau,
        config.convergence_tolerance,
    );

    let new_deviation1 = new_deviation(player_one_deviation, player_one_new_volatility, v1);
    let new_deviation2 = new_deviation(player_two_deviation, player_two_new_volatility, v2);

    let new_rating1 = new_rating(player_one_rating, new_deviation1, outcome1, g1, e1);
    let new_rating2 = new_rating(player_two_rating, new_deviation2, outcome2, g2, e2);

    // We return the new values, converted back to the original scale.
    let player_one_new = skillratings::glicko2::Glicko2Rating {
        rating: new_rating1.mul_add(173.7178, 1500.0),
        deviation: new_deviation1 * 173.7178,
        volatility: player_one_new_volatility,
    };
    let player_two_new = skillratings::glicko2::Glicko2Rating {
        rating: new_rating2.mul_add(173.7178, 1500.0),
        deviation: new_deviation2 * 173.7178,
        volatility: player_two_new_volatility,
    };

    (player_one_new, player_two_new)
}

fn g_value(deviation: f64) -> f64 {
    (1.0 + ((3.0 * deviation.powi(2)) / (std::f64::consts::PI.powi(2))))
        .sqrt()
        .recip()
}

fn e_value(rating: f64, opponent_rating: f64, g: f64) -> f64 {
    (1.0 + (-g * (rating - opponent_rating)).exp()).recip()
}

fn v_value(g: f64, e: f64) -> f64 {
    (g.powi(2) * e * (1.0 - e)).recip()
}

fn delta_value(outcome: f64, v: f64, g: f64, e: f64) -> f64 {
    v * (g * (outcome - e))
}

fn f_value(
    x: f64,
    delta_square: f64,
    deviation_square: f64,
    v: f64,
    volatility: f64,
    tau: f64,
) -> f64 {
    let i = (x.exp() * (delta_square - deviation_square - v - x.exp()))
        / (2.0 * (deviation_square + v + x.exp()).powi(2));

    let j = (x - volatility.powi(2).ln()) / tau.powi(2);

    i - j
}

fn new_volatility(
    old_volatility: f64,
    delta_squared: f64,
    deviation_squared: f64,
    v: f64,
    tau: f64,
    convergence_tolerance: f64,
) -> f64 {
    let mut a = old_volatility.powi(2).ln();
    let mut b = if delta_squared > deviation_squared + v {
        (delta_squared - deviation_squared - v).ln()
    } else {
        let mut k = 1.0;
        while f_value(
            a - k * tau,
            delta_squared,
            deviation_squared,
            v,
            old_volatility,
            tau,
        ) < 0.0
        {
            k += 1.0;
        }
        a - k * tau
    };

    let mut fa = f_value(a, delta_squared, deviation_squared, v, old_volatility, tau);
    let mut fb = f_value(b, delta_squared, deviation_squared, v, old_volatility, tau);

    // 0.000001 is the convergence tolerance suggested by Mark Glickman.
    while (b - a).abs() > convergence_tolerance {
        let c = a + ((a - b) * fa / (fb - fa));
        let fc = f_value(c, delta_squared, deviation_squared, v, old_volatility, tau);

        if fc * fb <= 0.0 {
            a = b;
            fa = fb;
        } else {
            fa /= 2.0;
        }

        b = c;
        fb = fc;
    }

    (a / 2.0).exp()
}

fn new_deviation(deviation: f64, new_volatility: f64, v: f64) -> f64 {
    let pre_deviation = deviation.hypot(new_volatility);

    ((pre_deviation.powi(2).recip()) + (v.recip()))
        .sqrt()
        .recip()
}

fn new_rating(rating: f64, new_deviation: f64, outcome: f64, g_value: f64, e_value: f64) -> f64 {
    (new_deviation.powi(2) * g_value).mul_add(outcome - e_value, rating)
}

// END MODIFIED CODE FROM https://crates.io/crates/skillratings
