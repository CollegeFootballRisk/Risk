/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![feature(drain_filter)]
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate serde_derive;
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

use structs::{
    Bar, PlayerMoves, Stats, TerritoryOwners, TerritoryOwnersInsert, TerritoryStats, TurnInfo,
    Victor,
};

#[must_use]
pub fn establish_connection() -> PgConnection {
    // Rocket figment gives us the information we need, then we discard it
    let database_url: String = rocket::build()
        .figment()
        .extract_inner("databases.postgres_global.url")
        .expect("Database not set in configuration.");
    PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

fn get_teams(territory_players: Vec<PlayerMoves>) -> Vec<i32> {
    let mut teams = territory_players
        .iter()
        .map(|x| x.team)
        .collect::<Vec<i32>>();
    teams.sort_unstable();
    teams.dedup();
    teams
}

fn determine_victor(lottery: f64, map: HashMap<i32, Victor>) -> i32 {
    let mut victorsum = 0_f64;
    //println!("Map: {:?}", map);
    let mut victor = 0;
    for (key, val) in &map {
        //println!("Key {:?} Val {:?}",key,val);
        victorsum += val.power;
        if lottery - victorsum < 0_f64 {
            victor = *key;
            break;
        }
    }
    victor
}

fn get_mvp(mut territory_players: Vec<PlayerMoves>) -> PlayerMoves {
    let rng = match territory_players.len() {
        1 => 0,
        _ => rand::thread_rng().gen_range(1..territory_players.len()),
    };
    territory_players.remove(rng)
}

fn process_territories(
    territories: Vec<TerritoryOwners>,
    mut players: Vec<PlayerMoves>,
) -> (
    Vec<TerritoryOwnersInsert>,
    Vec<PlayerMoves>,
    HashMap<i32, Stats>,
    Vec<TerritoryStats>,
) {
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
        let teams = get_teams(territory_players.clone());
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
                    territory: territory.territory_id,
                    ..TerritoryStats::default()
                });
                continue;
            }
            1 => {
                // Due to All-or-nothing, we don't get to just assume that this team gets it
                if territory_players
                    .iter()
                    .map(|mover| mover.power)
                    .sum::<f64>()
                    == 0.0
                {
                    // Then this is the same case as if there is no teams, next
                    dbg!("Team has no power");
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
                        territory: territory.territory_id,
                        ..TerritoryStats::default()
                    });
                    continue;
                }
                dbg!("One Team");
                let mvp = get_mvp(territory_players.clone());
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
                territory_players.iter().map(|mover| mover.power/mover.multiplier).sum::<f64>();
                */
                // add team stats
                handle_team_stats(&mut stats, territory_players.clone());
                territory_stats.push(TerritoryStats {
                    team: teams[0],
                    season: territory.season,
                    day: territory.day,
                    ones: territory_players
                        .iter()
                        .filter(|player| player.stars == 1)
                        .count() as i32,
                    twos: territory_players
                        .iter()
                        .filter(|player| player.stars == 2)
                        .count() as i32,
                    threes: territory_players
                        .iter()
                        .filter(|player| player.stars == 3)
                        .count() as i32,
                    fours: territory_players
                        .iter()
                        .filter(|player| player.stars == 4)
                        .count() as i32,
                    fives: territory_players
                        .iter()
                        .filter(|player| player.stars == 5)
                        .count() as i32,
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
                // Due to All-or-nothing, we don't get to just assume that this team gets it
                if territory_players
                    .iter()
                    .map(|mover| mover.power)
                    .sum::<f64>()
                    == 0.0
                {
                    // Then this is the same case as if there is no teams, next
                    dbg!("Team has no power");
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
                        territory: territory.territory_id,
                        ..TerritoryStats::default()
                    });
                    continue;
                }

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
                    map.insert(team, Victor::default()); // stars, power, ones, twos, threes, fours, fives
                }

                for player in &territory_players {
                    //dbg!(player.id, player.team);
                    if player.alt_score >= 175 {
                        continue;
                    }
                    map.get_mut(&player.team)
                        .unwrap()
                        .power(player.power)
                        .stars(player.stars);
                }

                let totalpower: f64 = map.values().map(|x| (x.power)).sum();
                //dbg!(totalpower);
                let lottery = ChaCha12Rng::from_entropy().gen_range(0_f64..totalpower);

                let victor = determine_victor(lottery, map.clone());

                //dbg!("Victor: {}",victor);
                let territory_victors = territory_players
                    .clone()
                    .drain_filter(|player| player.team == victor)
                    .collect::<Vec<_>>();
                let mvp = get_mvp(territory_victors);
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

                let total_power = territory_players
                    .iter()
                    .map(|mover| mover.power as f64)
                    .sum::<f64>();
                handle_team_stats(&mut stats, territory_players);
                for (key, val) in &map {
                    territory_stats.push(TerritoryStats {
                        team: *key,
                        season: territory.season,
                        day: territory.day,
                        ones: val.ones,
                        twos: val.twos,
                        threes: val.threes,
                        fours: val.fours,
                        fives: val.fives,
                        teampower: val.power,
                        chance: val.power / total_power,
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

fn handle_team_stats(stats: &mut HashMap<i32, Stats>, territory_players: Vec<PlayerMoves>) {
    //dbg!("wallop");
    //dbg!(&territory_players.len());
    for i in territory_players {
        stats
            .entry(i.team)
            .or_insert_with(|| Stats::new(i.season * 1000 + i.day + 1, i.season, i.day, i.team))
            .starpower(i.power / i.multiplier)
            .effectivepower(i.power.round() as f64)
            .add_player_or_merc(i.merc)
            .stars(i.stars);
    }
}

#[cfg(feature = "risk_image")]
fn make_image(territories: Vec<TerritoryOwnersInsert>, conn: &PgConnection) {
    use crate::structs::Team;
    extern crate image;
    extern crate nsvg;
    // first we got get the SVG image
    use std::fs;
    let teams = Team::load(conn);
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
                    &base.replace('?', &item.territory_id.to_string()),
                    team_map.get(&item.owner_id).unwrap(),
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

fn user_update(
    turninfoblock: &TurnInfo,
    conn: &PgConnection,
) -> Result<Vec<Bar>, diesel::result::Error> {
    let query = format!(
        "SELECT do_user_update({},{})",
        &turninfoblock.day.unwrap().to_string(),
        &turninfoblock.season.unwrap().to_string()
    );
    sql_query(query).load(conn)
}

fn main() {
    use std::time::Instant;
    let now = Instant::now();
    let state = runtime();
    let elapsed = now.elapsed();
    let end = Instant::now();
    println!("Elapsed: {:.2?}", elapsed);
    println!("Start Time: {:.2?}", now);
    println!("End Time: {:.2?}", end);
    std::process::exit(match state {
        Ok(_) => 0,
        Err(err) => {
            eprintln!("error: {:?}", err);
            1
        }
    });
}

fn runtime() -> Result<(), diesel::result::Error> {
    // Connect to the Postgres DB
    let conn: PgConnection = establish_connection();
    // Get the active turn
    // start_time_now then sets the start time to the current time.
    let mut turninfoblock = TurnInfo::get_latest(&conn)?;
    turninfoblock.start_time_now();
    //dbg!(&turninfoblock.season, &turninfoblock.day);
    // Now we go get all player moves for the current day
    let players = PlayerMoves::load(
        &turninfoblock.season.unwrap(),
        &turninfoblock.day.unwrap(),
        &conn,
    )?;
    // And a list of all territories, and their current owners:
    let territories = TerritoryOwners::load(
        &turninfoblock.season.unwrap(),
        &turninfoblock.day.unwrap(),
        &conn,
    )?;
    // If there are no moves to load, we'll exit as something's not right.
    // TODO: Return Err, not Ok
    if players.is_empty() {
        return Ok(());
    }
    let move_ids = players.iter().map(|x| x.id).collect::<Vec<i32>>();
    let min_value = move_ids.iter().min();
    let max_value = move_ids.iter().max();
    let (owners, mvps, stats, territory_stats) = process_territories(territories, players);
    TerritoryStats::insert(territory_stats, &conn)?;
    Stats::insert(stats, turninfoblock.id, &conn)?;
    let territory_insert = TerritoryOwnersInsert::insert(&owners, &conn)?;
    dbg!(territory_insert);
    let playermoves = PlayerMoves::mvps(mvps, &conn)?;
    dbg!(playermoves);
    let mergemoves =
        PlayerMoves::mergemoves(*min_value.unwrap_or(&0), *max_value.unwrap_or(&0), &conn)?;
    dbg!(mergemoves);

    // This calls an SQL function that updates each user's statistics
    // Not ideal, TODO: we ought to implement this in Rust.
    let userupdate = user_update(&turninfoblock, &conn);
    match userupdate {
        Ok(ok) => println!("Users updated successfully {}", ok[0].do_user_update),
        Err(e) => println!("Failed to update users: {:?}", e),
    }
    turninfoblock.rollendtime = Some(Utc::now().naive_utc());
    turninfoblock.complete = Some(true);
    turninfoblock.active = Some(false);
    match TurnInfo::update_or_insert(&turninfoblock, &conn) {
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
            println!("Create new turn succeeded")
        }
        Err(e) => println!("Failed to make new turn {:?}", e),
    }

    #[cfg(feature = "risk_image")]
    make_image(owners, &conn);
    Ok(())
}
