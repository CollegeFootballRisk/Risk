#![feature(drain_filter)]
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate serde_derive;
extern crate dotenv;
extern crate rand;
extern crate rand_chacha;
pub mod schema;
pub mod structs;

use chrono::Utc;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::sql_query;
use rand::prelude::*;
use rand_chacha::ChaCha12Rng;
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::Read;
use std::io::Write;

use structs::{
    PlayerMoves, Stats, TerritoryOwners, TerritoryOwnersInsert, TerritoryStats, TurnInfo,
};

#[must_use] pub fn establish_connection() -> PgConnection {
    dotenv::from_filename(".env").ok();
    let database_url = dotenv::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

fn getteams(territory_players: Vec<PlayerMoves>) -> Vec<i32> {
    let mut teams = territory_players.iter().map(|x| x.team).collect::<Vec<i32>>();
    teams.sort_unstable();
    teams.dedup();
    teams
}

fn determinevictor(lottery: f64, map: HashMap<i32, (i32, f64, i32, i32, i32, i32, i32)>) -> i32 {
    let mut victorsum = 0_f64;
    //println!("Map: {:?}", map);
    let mut victor = 0;
    for (key, val) in &map {
        //println!("Key {:?} Val {:?}",key,val);
        victorsum += val.1;
        if lottery - victorsum < 0_f64 {
            victor = *key;
            break;
        }
    }
    victor
}

fn getmvp(mut territory_players: Vec<PlayerMoves>) -> PlayerMoves {
    let rng = match territory_players.len() {
        1 => 0,
        _ => rand::thread_rng().gen_range(1..territory_players.len()),
    };
    territory_players.remove(rng)
}

fn process_territories(
    territories: Vec<TerritoryOwners>,
    mut players: Vec<PlayerMoves>,
) -> (Vec<TerritoryOwnersInsert>, Vec<PlayerMoves>, HashMap<i32, Stats>, Vec<TerritoryStats>) {
    dbg!("process_territories");
    dbg!(territories.len());
    let mut new_owners: Vec<TerritoryOwnersInsert> = Vec::new();
    let mut mvps: Vec<PlayerMoves> = Vec::new();
    let mut stats: HashMap<i32, Stats> = HashMap::new();
    let mut territory_stats: Vec<TerritoryStats> = Vec::new();
    for territory in territories {
        dbg!(&territory.territory_id);
        let territory_players = players
            .drain_filter(|player| player.territory == territory.territory_id)
            .collect::<Vec<_>>();
        dbg!(&territory_players.len());
        let teams = getteams(territory_players.clone());
        match teams.len() {
            0 => {
                dbg!("Zero Team");
                new_owners.push(TerritoryOwnersInsert {
                    territory_id: territory.territory_id,
                    territory_name: None,
                    owner_id: territory.owner_id,
                    day: territory.day + 1,
                    season: territory.season,
                    previous_owner_id: territory.owner_id,
                    random_number: 0_f64,
                    mvp: Some(0),
                });
                // add team territory count to stats
                stats
                    .entry(territory.owner_id)
                    .or_insert_with(|| {
                        Stats::new(
                            territory.season * 1000 + territory.season + 1,
                            territory.season,
                            territory.day,
                            territory.owner_id,
                        )
                    })
                    .territorycount += 1;

                territory_stats.push(TerritoryStats {
                    team: territory.owner_id,
                    season: territory.season,
                    day: territory.day,
                    ones: 0,
                    twos: 0,
                    threes: 0,
                    fours: 0,
                    fives: 0,
                    teampower: 0.0,
                    chance: 1.00,
                    territory: territory.territory_id,
                    territory_power: 0.00,
                });
                continue;
            }
            1 => {
                dbg!("One Team");
                let mvp = getmvp(territory_players.clone());
                mvps.push(mvp.clone());
                new_owners.push(TerritoryOwnersInsert {
                    territory_id: territory.territory_id,
                    territory_name: None,
                    owner_id: teams[0],
                    day: territory.day + 1,
                    season: territory.season,
                    previous_owner_id: territory.owner_id,
                    random_number: 0_f64,
                    mvp: Some(mvp.user_id),
                });
                stats
                    .entry(teams[0])
                    .or_insert_with(|| {
                        Stats::new(
                            territory.season * 1000 + territory.season + 1,
                            territory.season,
                            territory.day,
                            teams[0],
                        )
                    })
                    .territorycount += 1;
                stats
                    .entry(territory.owner_id)
                    .or_insert_with(|| {
                        Stats::new(
                            territory.season * 1000 + territory.season + 1,
                            territory.season,
                            territory.day,
                            territory.owner_id,
                        )
                    })
                    .territorycount += 0;
                /*stats
                .entry(teams[0])
                .or_insert_with(|| {
                    Stats::new(
                        territory.season * 1000 + territory.season + 1,
                        territory.season,
                        territory.day,
                        teams[0],
                    )
                })
                .starpower +=
                territory_players.iter().map(|mover| mover.power/mover.multiplier.unwrap_or(1.0)).sum::<f64>();*/
                // add team stats
                handleteamstats(&mut stats, territory_players.clone());
                territory_stats.push(TerritoryStats {
                    team: teams[0],
                    season: territory.season,
                    day: territory.day,
                    ones: territory_players.iter().filter(|player| player.stars == 1).count()
                        as i32,
                    twos: territory_players.iter().filter(|player| player.stars == 2).count()
                        as i32,
                    threes: territory_players.iter().filter(|player| player.stars == 3).count()
                        as i32,
                    fours: territory_players.iter().filter(|player| player.stars == 4).count()
                        as i32,
                    fives: territory_players.iter().filter(|player| player.stars == 5).count()
                        as i32,
                    teampower: territory_players
                        .iter()
                        .map(|mover| mover.power as f64)
                        .sum::<f64>(),
                    chance: 1.00,
                    territory: territory.territory_id,
                    territory_power: territory_players
                        .iter()
                        .map(|mover| mover.power as f64)
                        .sum::<f64>(),
                });
                continue;
            }
            _ => {
                dbg!(&teams);
                let mut map = HashMap::new();
                stats
                    .entry(territory.owner_id)
                    .or_insert_with(|| {
                        Stats::new(
                            territory.season * 1000 + territory.season + 1,
                            territory.season,
                            territory.day,
                            territory.owner_id,
                        )
                    })
                    .territorycount += 0;
                for team in teams {
                    map.insert(team, (0, 0_f64, 0, 0, 0, 0, 0)); // stars, power, ones, twos, threes, fours, fives
                }

                for player in &territory_players {
                    //dbg!(player.id, player.team);
                    if player.alt_score >= 175 {
                        continue;
                    } else {
                        map.get_mut(&player.team).unwrap().0 += player.stars;
                        map.get_mut(&player.team).unwrap().1 += player.power;
                        //dbg!(player);
                        match player.stars {
                            1 => {
                                map.get_mut(&player.team).unwrap().2 += 1;
                            }
                            2 => {
                                map.get_mut(&player.team).unwrap().3 += 1;
                            }
                            3 => {
                                map.get_mut(&player.team).unwrap().4 += 1;
                            }
                            4 => {
                                map.get_mut(&player.team).unwrap().5 += 1;
                            }
                            5 => {
                                map.get_mut(&player.team).unwrap().6 += 1;
                            }
                            _ => {
                                dbg!("unknown stars");
                            }
                        }
                    }
                }

                let totalpower: f64 = map.values().map(|x| (x.1)).sum();
                //dbg!(totalpower);
                let lottery = ChaCha12Rng::from_entropy().gen_range(0_f64..totalpower);

                let victor = determinevictor(lottery, map.clone());

                //dbg!("Victor: {}",victor);
                let territory_victors = territory_players
                    .clone()
                    .drain_filter(|player| player.team == victor)
                    .collect::<Vec<_>>();
                let mvp = getmvp(territory_victors);
                new_owners.push(TerritoryOwnersInsert {
                    territory_id: territory.territory_id,
                    territory_name: None,
                    owner_id: victor,
                    day: territory.day + 1,
                    season: territory.season,
                    previous_owner_id: territory.owner_id,
                    random_number: lottery,
                    mvp: Some(mvp.user_id),
                });

                stats
                    .entry(victor)
                    .or_insert_with(|| {
                        Stats::new(
                            territory.season * 1000 + territory.season + 1,
                            territory.season,
                            territory.day,
                            victor,
                        )
                    })
                    .territorycount += 1;

                let total_power =
                    territory_players.iter().map(|mover| mover.power as f64).sum::<f64>();
                handleteamstats(&mut stats, territory_players);
                for (key, val) in &map {
                    territory_stats.push(TerritoryStats {
                        team: *key,
                        season: territory.season,
                        day: territory.day,
                        ones: val.2,
                        twos: val.3,
                        threes: val.4,
                        fours: val.5,
                        fives: val.6,
                        teampower: val.1,
                        chance: val.1 / total_power,
                        territory: territory.territory_id,
                        territory_power: total_power,
                    });
                }
                mvps.push(mvp);
            }
        }
    }
    (new_owners, mvps, stats, territory_stats)
}

fn handleteamstats(stats: &mut HashMap<i32, Stats>, territory_players: Vec<PlayerMoves>) {
    //dbg!("wallop");
    //dbg!(&territory_players.len());
    for i in territory_players {
        stats
            .entry(i.team)
            .or_insert_with(|| Stats::new(i.season * 1000 + i.day + 1, i.season, i.day, i.team))
            .starpower += i.power / i.multiplier;
        stats
            .entry(i.team)
            .or_insert_with(|| Stats::new(i.season * 1000 + i.day + 1, i.season, i.day, i.team))
            .effectivepower += i.power.round() as f64;

        if i.merc {
            stats
                .entry(i.team)
                .or_insert_with(|| Stats::new(i.season * 1000 + i.day + 1, i.season, i.day, i.team))
                .merccount += 1;
        } else {
            stats
                .entry(i.team)
                .or_insert_with(|| Stats::new(i.season * 1000 + i.day + 1, i.season, i.day, i.team))
                .playercount += 1;
        }
        match i.stars {
            1 => {
                stats
                    .entry(i.team)
                    .or_insert_with(|| {
                        Stats::new(i.season * 1000 + i.day + 1, i.season, i.day, i.team)
                    })
                    .ones += 1;
            }
            2 => {
                stats
                    .entry(i.team)
                    .or_insert_with(|| {
                        Stats::new(i.season * 1000 + i.day + 1, i.season, i.day, i.team)
                    })
                    .twos += 1;
            }
            3 => {
                stats
                    .entry(i.team)
                    .or_insert_with(|| {
                        Stats::new(i.season * 1000 + i.day + 1, i.season, i.day, i.team)
                    })
                    .threes += 1;
            }
            4 => {
                stats
                    .entry(i.team)
                    .or_insert_with(|| {
                        Stats::new(i.season * 1000 + i.day + 1, i.season, i.day, i.team)
                    })
                    .fours += 1;
            }
            5 => {
                stats
                    .entry(i.team)
                    .or_insert_with(|| {
                        Stats::new(i.season * 1000 + i.day + 1, i.season, i.day, i.team)
                    })
                    .fives += 1;
            }
            _ => {
                stats
                    .entry(i.team)
                    .or_insert_with(|| {
                        Stats::new(i.season * 1000 + i.day + 1, i.season, i.day, i.team)
                    })
                    .ones += 1;
            }
        }
    }
}

#[cfg(feature = "risk_image")]
fn make_image(territories: Vec<TerritoryOwnersInsert>, conn: &PgConnection) {
    use crate::structs::Team;
    extern crate image;
    extern crate nsvg;
    // first we got get the SVG image
    use std::fs;
    let teams = Team::load(&conn);
    let mut vec = fs::read_to_string("resources/map.svg").unwrap();
    let base: String = "{{?}}".to_owned();
    let mut team_map = HashMap::new();
    match teams {
        Ok(teams) => {
            for team in teams {
                team_map.insert(team.id, team.color);
            }
            for item in territories {
                vec = vec.replace(
                    &base.replace("?", &item.territory_id.to_string()),
                    &team_map.get(&item.owner_id).unwrap(),
                );
            }
            let svg = nsvg::parse_str(&vec, nsvg::Units::Pixel, 96.0).unwrap();
            let image = svg.rasterize(2.0).unwrap();
            let (width, height) = image.dimensions();
            image::save_buffer(
                "../server/static/images/curr_map.png",
                &image.into_raw(),
                width,
                height,
                image::ColorType::Rgba8,
            )
            .expect("Failed to save png.");
        }
        Err(e) => {
            dbg!(e);
        }
    }
}

fn main() {
    use std::time::Instant;
    let now = Instant::now();
    {
        let conn: PgConnection = establish_connection();
        let turninfodata = TurnInfo::get_latest(&conn);
        let mut turninfoblock = turninfodata.unwrap();
        turninfoblock.rollstarttime = Some(Utc::now().naive_utc());
        dbg!(&turninfoblock.season, &turninfoblock.day);
        let players =
            PlayerMoves::load(&turninfoblock.season.unwrap(), &turninfoblock.day.unwrap(), &conn);
        let territories = TerritoryOwners::load(
            &turninfoblock.season.unwrap(),
            &turninfoblock.day.unwrap(),
            &conn,
        );
        let result = match territories {
            Ok(territories) => {
                match players {
                    Ok(players) => {
                        if !players.is_empty() {
                            let move_ids = players.iter().map(|x| x.id).collect::<Vec<i32>>();
                            let min_value = move_ids.iter().min();
                            let max_value = move_ids.iter().max();
                            //dbg!(min_value, max_value, players.len());
                            let (owners, mvps, stats, territory_stats) =
                                process_territories(territories, players);
                            //dbg!(&owners,&mvps, &stats);
                            //dbg!(&stats);
                            match TerritoryStats::insert(territory_stats, &conn) {
                                Ok(_result) => {
                                    match Stats::insert(stats, turninfoblock.id, &conn) {
                                        Ok(_result) => {
                                            match TerritoryOwnersInsert::insert(&owners, &conn) {
                                                Ok(result) => {
                                                    dbg!(result);
                                                    match PlayerMoves::mvps(mvps, &conn) {
                                                        Ok(result) => {
                                                            dbg!(result);
                                                            match PlayerMoves::mergemoves(
                                                                *min_value.unwrap_or(&0),
                                                                *max_value.unwrap_or(&0),
                                                                &conn,
                                                            ) {
                                                                Ok(result) => {
                                                                    dbg!(result);
                                                                    use diesel::sql_types::Bool;
                                                                    #[derive(QueryableByName)]
                                                                    struct Bar {
                                                                        #[sql_type = "Bool"]
                                                                        do_user_update: bool,
                                                                    }
                                                                    let query = format!(
                                                                        "SELECT do_user_update({},{})",
                                                                        &turninfoblock.day.unwrap().to_string(),
                                                                        &turninfoblock.season.unwrap().to_string()
                                                                    );
                                                                    let userupdate: Result<
                                                                        Vec<Bar>,
                                                                        diesel::result::Error,
                                                                    > = sql_query(query)
                                                                        .load(&conn);
                                                                    match userupdate {
                                                                        Ok(ok) => println!(
                                                                            "Users updated successfully {}",
                                                                            ok[0].do_user_update.to_string()
                                                                        ),
                                                                        Err(e) => println!(
                                                                            "Failed to update users: {:?}",
                                                                            e
                                                                        ),
                                                                    }
                                                                    turninfoblock.rollendtime =
                                                                        Some(
                                                                            Utc::now().naive_utc(),
                                                                        );
                                                                    turninfoblock.complete =
                                                                        Some(true);
                                                                    turninfoblock.active =
                                                                        Some(false);
                                                                    match TurnInfo::update_or_insert(
                                                                        &turninfoblock,
                                                                        &conn,
                                                                    ) {
                                                                        Ok(_ok) => {
                                                                            println!("Update turninfo success.")
                                                                        }
                                                                        Err(_e) => {
                                                                            println!("Error updating turninfo.")
                                                                        }
                                                                    }
                                                                    match TurnInfo::insert_new(
                                                                        turninfoblock.season.unwrap(),
                                                                        turninfoblock.day.unwrap() + 1,
                                                                        true,
                                                                        false,
                                                                        &conn,
                                                                    ) {
                                                                        Ok(_ok) => {
                                                                            println!("Created new turn succeeded")
                                                                        }
                                                                        Err(e) => println!(
                                                                            "Failed to make new turn {:?}",
                                                                            e
                                                                        ),
                                                                    }
                                                                    let f_in = OpenOptions::new()
                                                                        .read(true)
                                                                        .write(true)
                                                                        .create(true)
                                                                        .open(".env");
                                                                    let f_out = OpenOptions::new()
                                                                        .create(true)
                                                                        .write(true)
                                                                        .open(".env");
                                                                    let mut buffer = String::new();
                                                                    let mut f_in = f_in.unwrap();
                                                                    match f_in
                                                                        .read_to_string(&mut buffer)
                                                                    {
                                                                        Ok(_ok) => {
                                                                            dbg!(
                                                                                "Read String OKAY"
                                                                            );
                                                                        }
                                                                        Err(e) => {
                                                                            dbg!("Error", e);
                                                                        }
                                                                    }
                                                                    dotenv::from_filename(".env")
                                                                        .ok();
                                                                    let day =
                                                                        dotenv::var("day").unwrap();
                                                                    let new_day = &day
                                                                        .parse::<i32>()
                                                                        .unwrap()
                                                                        + 1;
                                                                    buffer = buffer.replace(
                                                                        &format!("day={}", day),
                                                                        &format!(
                                                                            "day={}",
                                                                            new_day.to_string()
                                                                        )[..],
                                                                    );
                                                                    f_out
                                                                        .unwrap()
                                                                        .write_all(
                                                                            buffer.as_bytes(),
                                                                        )
                                                                        .expect("error");

                                                                    #[cfg(feature = "risk_image")]
                                                                    make_image(owners, &conn);
                                                                }
                                                                Err(e) => {
                                                                    dbg!(e);
                                                                }
                                                            }
                                                        }
                                                        Err(e) => {
                                                            dbg!(e);
                                                        }
                                                    }
                                                }
                                                Err(e) => {
                                                    dbg!(e);
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            dbg!(e);
                                        }
                                    }
                                }
                                Err(e) => {
                                    dbg!(e);
                                }
                            }
                        }
                        "Okay".to_string()
                    }
                    Err(e) => e.to_string(),
                }
            }
            Err(e) => e.to_string(),
        };

        println!("Result: {}", result);
    }
    let elapsed = now.elapsed();
    let end = Instant::now();
    println!("Elapsed: {:.2?}", elapsed);
    println!("Start Time: {:.2?}", now);
    println!("End Time: {:.2?}", end);
}
