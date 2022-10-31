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
pub mod optional;
pub mod schema;
pub mod structs;

use chrono::{Duration, NaiveTime,DateTime,Utc,Datelike,Timelike, NaiveDateTime};
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::sql_query;
use rand::prelude::*;
use rand_chacha::ChaCha12Rng;
use std::collections::HashMap;

const ALT_CUTOFF: i32 = 75;
const AON_END: i32 = 48;
const AON_START: i32 = 3;

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

/// Based on random number `lottery`, return the ID of the MVP user
fn determine_victor(lottery: f64, map: HashMap<i32, Victor>) -> i32 {
    let mut victorsum = 0_f64;
    let mut victor = 0;
    for (key, val) in &map {
        victorsum += val.power;
        if lottery - victorsum < 0_f64 {
            victor = *key;
            break;
        }
    }
    victor
}

fn get_mvp(mut territory_players: Vec<PlayerMoves>) -> PlayerMoves {
    territory_players.retain(|x| x.alt_score >= ALT_CUTOFF);
    let rng = match territory_players.len() {
        1 => 0,
        _ => rand::thread_rng().gen_range(0..territory_players.len()),
    };
    territory_players.remove(rng)
}

/// Go territory by territory and determine new owner, MVP, and statistics
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
                new_owners.push(TerritoryOwnersInsert::new(
                    &territory,
                    territory.owner_id,
                    None,
                    None,
                ));
                // add team territory count to stats
                stats
                    .entry(territory.owner_id)
                    .or_insert_with(|| Stats::new(territory.turn_id + 1, territory.owner_id))
                    .territorycount += 1;
                territory_stats.push(TerritoryStats {
                    team: territory.owner_id,
                    turn_id: territory.turn_id,
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
                    new_owners.push(TerritoryOwnersInsert::new(
                        &territory,
                        territory.owner_id,
                        None,
                        None,
                    ));
                    // add team territory count to stats
                    stats
                        .entry(territory.owner_id)
                        .or_insert_with(|| Stats::new(territory.turn_id + 1, territory.owner_id))
                        .territorycount += 1;

                    territory_stats.push(TerritoryStats {
                        team: territory.owner_id,
                        turn_id: territory.turn_id,
                        territory: territory.territory_id,
                        ..TerritoryStats::default()
                    });
                    continue;
                }
                dbg!("One Team");
                let mvp = get_mvp(territory_players.clone());
                mvps.push(mvp.clone());
                new_owners.push(TerritoryOwnersInsert::new(
                    &territory,
                    teams[0],
                    None,
                    Some(mvp.user_id),
                ));
                stats
                    .entry(teams[0])
                    .or_insert_with(|| Stats::new(territory.turn_id + 1, teams[0]))
                    .territorycount += 1;
                stats
                    .entry(territory.owner_id)
                    .or_insert_with(|| Stats::new(territory.turn_id + 1, territory.owner_id))
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
                // This team might be dead, push to the odds table
                if teams[0] != territory.owner_id {
                    territory_stats.push(TerritoryStats {
                        team: territory.owner_id,
                        turn_id: territory.turn_id,
                        territory: territory.territory_id,
                        territory_power: territory_players
                            .iter()
                            .map(|mover| mover.power as f64)
                            .sum::<f64>(),
                        chance: 0.00,
                        ..TerritoryStats::default()
                    });
                }
                territory_stats.push(TerritoryStats {
                    team: teams[0],
                    turn_id: territory.turn_id,
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
                    new_owners.push(TerritoryOwnersInsert::new(
                        &territory,
                        territory.owner_id,
                        None,
                        None,
                    ));
                    // add team territory count to stats
                    stats
                        .entry(territory.owner_id)
                        .or_insert_with(|| Stats::new(territory.turn_id + 1, territory.owner_id))
                        .territorycount += 1;

                    territory_stats.push(TerritoryStats {
                        team: territory.owner_id,
                        turn_id: territory.turn_id,
                        territory: territory.territory_id,
                        ..TerritoryStats::default()
                    });
                    continue;
                }

                let mut map = HashMap::new();
                stats
                    .entry(territory.owner_id)
                    .or_insert_with(|| Stats::new(territory.turn_id + 1, territory.owner_id))
                    .territorycount += 0;
                for team in teams {
                    map.insert(team, Victor::default()); // stars, power, ones, twos, threes, fours, fives
                }

                for player in &territory_players {
                    //dbg!(player.id, player.team);
                    if player.alt_score >= ALT_CUTOFF {
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
                new_owners.push(TerritoryOwnersInsert::new(
                    &territory,
                    victor,
                    Some(lottery),
                    Some(mvp.user_id),
                ));

                stats
                    .entry(victor)
                    .or_insert_with(|| Stats::new(territory.turn_id + 1, victor))
                    .territorycount += 1;

                let total_power = territory_players
                    .iter()
                    .map(|mover| mover.power as f64)
                    .sum::<f64>();
                handle_team_stats(&mut stats, territory_players);
                for (key, val) in &map {
                    territory_stats.push(TerritoryStats {
                        team: *key,
                        turn_id: territory.turn_id,
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
                // Also check if owning team needs spanked:
                if !map.contains_key(&territory.owner_id) {
                    territory_stats.push(TerritoryStats {
                        team: territory.owner_id,
                        turn_id: territory.turn_id,
                        territory: territory.territory_id,
                        territory_power: total_power,
                        chance: 0.00,
                        ..TerritoryStats::default()
                    });
                }
            }
        }
    }
    (new_owners, mvps, stats, territory_stats)
}

fn handle_team_stats(stats: &mut HashMap<i32, Stats>, territory_players: Vec<PlayerMoves>) {
    for i in territory_players {
        stats
            .entry(i.team)
            .or_insert_with(|| Stats::new(i.turn_id + 1, i.team))
            .starpower(i.power / i.multiplier)
            .effectivepower(i.power.round() as f64)
            .add_player_or_merc(i.merc)
            .stars(i.stars);
    }
}

/// Updates the statistics for all users after the roll
fn user_update(
    turninfoblock: &TurnInfo,
    conn: &PgConnection,
) -> Result<Vec<Bar>, diesel::result::Error> {
    let query = format!(
        "SELECT do_user_update({},{})",
        &turninfoblock.day.to_string(),
        &turninfoblock.season.to_string()
    );
    sql_query(query).load(conn)
}

/// Updates chaos bridges randomly
#[cfg(feature = "chaos")]
fn chaos_update(
    territories: &[TerritoryOwnersInsert],
    turn_id_n: i32,
    settings: &rocket::figment::Figment,
    conn: &PgConnection,
) -> Result<(), diesel::result::Error> {
    use crate::schema::territory_adjacency;
    // First, get the maximum and minimum territory numbers
    let max_territory: i32 = territories
        .iter()
        .max_by(|a, b| a.territory_id.cmp(&b.territory_id))
        .map(|k| k.territory_id)
        .unwrap_or(-1);
    let min_territory: i32 = territories
        .iter()
        .min_by(|a, b| a.territory_id.cmp(&b.territory_id))
        .map(|k| k.territory_id)
        .unwrap_or(-2);
    // Decide how many bridges to add
    // First, read config for Max/Min:
    let max_bridges: u32 = settings
        .extract_inner("risk.max_chaos_bridges")
        .unwrap_or(5);
    let min_bridges: u32 = settings
        .extract_inner("risk.min_chaos_bridges")
        .unwrap_or(1);
    let chaos_territory_id: i32 = settings
        .extract_inner("risk.chaos_territory_id")
        .unwrap_or(999);
    let chaos_bridges_twoway: bool = settings
        .extract_inner("risk.chaos_bridges_twoway")
        .unwrap_or(false);
    dbg!(
        max_territory,
        min_territory,
        max_bridges,
        min_bridges,
        chaos_bridges_twoway,
        chaos_territory_id
    );
    // Random!
    // NOTE: THIS IS [low, high)
    let num: u32 = rand::thread_rng().gen_range(min_bridges..max_bridges);
    // Remove old bridges with note 'chaos_auto_managed'
    //diesel::delete(territory_adjacency::table.filter(note.eq("chaos_auto_managed")))
    //    .execute(conn)?;
    // Add new bridges with note 'chaos_auto_managed'
    // Goes 0, 1, 2, 3, num-1; excludes num just like normal languages
    let mut new_stuff = Vec::new();
    #[derive(Insertable)]
    #[table_name = "territory_adjacency"]
    struct TerritoryAdjacent<'a> {
        territory_id: i32,
        adjacent_id: i32,
        note: &'a str,
        min_turn: i32,
        max_turn: i32,
    }
    dbg!(num);
    for _ in 0..num {
        // Select random teritory id:
        let territory: i32 = rand::thread_rng().gen_range(min_territory..(max_territory + 1));
        new_stuff.push(TerritoryAdjacent {
            territory_id: chaos_territory_id,
            adjacent_id: territory,
            note: "chaos_auto_managed",
            min_turn: turn_id_n - 1,
            max_turn: turn_id_n,
        });
        if chaos_bridges_twoway {
            new_stuff.push(TerritoryAdjacent {
                territory_id: territory,
                adjacent_id: chaos_territory_id,
                note: "chaos_auto_managed",
                min_turn: turn_id_n - 1,
                max_turn: turn_id_n,
            });
        }
    }
    diesel::insert_into(territory_adjacency::table)
        .values(&new_stuff)
        .execute(conn)?;
    Ok(())
}

#[allow(dead_code)]
fn do_playoffs() {
    // If we have playoffs. then we need to cast off a new day
    // Steps:
    // 1. For each bracket, determine the winner
    // 1a. If a bracket has a tie, then randomly declare a winner
    // 2. Create new territory ownership by bracket
    // 3. Create new statistics, giving each team equal territories and zero players
    // 4. Pop on the new day
    // Because we're not technically carrying out a new day, we don't add anything to any player account.
}

fn next_roll(settings: &rocket::figment::Figment) -> NaiveDateTime {
    // Calculate new starttime
    let next_time = settings
        .extract_inner("risk.time")
        .unwrap_or("04:00:00");
    let naive_time = NaiveTime::parse_from_str(next_time, "%H:%M:%S").unwrap();
    let next_days = settings.extract_inner("risk.days").unwrap_or([1,2,3,4,5,6,7]);
    return next_day_in_seq(&next_days, &naive_time, &Utc::now());
}

// Function assumes that we're after today's roll
fn next_day_in_seq(next_days: &[i64], next_time: &NaiveTime, now: &DateTime<Utc>) -> NaiveDateTime{
    let curr_day: i64 = now.weekday().number_from_monday() as i64;
    let index: i64 = if next_days.len() == 0 && curr_day < 7 {(curr_day + 1) as i64}
    else if next_days.len() == 0 {1 as i64}
    else {
        if let Some(next) = next_days.iter().find(|&x| *x > curr_day) {*next as i64} else {next_days[0] as i64}
    };
    return (*now + Duration::days(index)).date().and_hms(next_time.hour(), next_time.minute(), next_time.second()).naive_utc()
}

fn runtime() -> Result<(), diesel::result::Error> {
    let rocket = rocket::build();
    let settings = rocket.figment();
    // Connect to the Postgres DB
    let conn: PgConnection = establish_connection();
    // Get the active turn
    // start_time_now then sets the start time to the current time.
    let mut turninfoblock = TurnInfo::get_latest(&conn)?;
    turninfoblock.start_time_now();
    //dbg!(&turninfoblock.season, &turninfoblock.day);
    // Now we go get all player moves for the current day
    let players = PlayerMoves::load(&turninfoblock.id, &conn)?;
    // And a list of all territories, and their current owners:
    let territories = TerritoryOwners::load(&turninfoblock.id, &conn)?;
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
    let aone = (turninfoblock.allornothingenabled == Some(true)
        || (turninfoblock.day + 1) >= AON_START)
        && (turninfoblock.day + 1) < AON_END;
    match TurnInfo::insert_new(
        turninfoblock.season,
        turninfoblock.day + 1,
        true,
        false,
        turninfoblock.map,
        aone,
        next_roll(&settings),
        &conn,
    ) {
        Ok(_ok) => {
            println!("Create new turn succeeded")
        }
        Err(e) => println!("Failed to make new turn {:?}", e),
    }

    #[cfg(feature = "risk_image")]
    optional::image::make_image(&owners, &conn);

    #[cfg(feature = "chaos")]
    {
        match chaos_update(&owners, turninfoblock.id + 1, &settings, &conn) {
            Ok(_) => println!("Chaos bridges updated."),
            Err(e) => println!("Chaos bridges couldn't update. \n Error: {:?}", e),
        }
    }

    Ok(())
}

/// Main function; runs RNG to determine new day parameters.
fn main() {
    use std::time::Instant;
    // Set up variables to know timing
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

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    use chrono::{NaiveDate,NaiveTime};

    #[test]
    fn test_next_day_in_seq() {
    let next_time = String::from("04:00:00");
    let naive_time = NaiveTime::parse_from_str(&next_time, "%H:%M:%S").unwrap();
    let mut next_days = [1,2,3,4,5,6,7];
    next_days.sort();
     
    let now = NaiveDate::from_ymd(2022, 10, 30).and_hms(4, 01, 01).into();
    let next = NaiveDate::from_ymd(2022, 10, 31).and_hms(4, 00, 00);
    assert_eq!(next, next_day_in_seq(&next_days, &naive_time, &DateTime::from_utc(now, Utc)));
    }

    #[test]
    fn test_next_day_in_seq_skip() {
    let next_time = String::from("04:00:00");
    let naive_time = NaiveTime::parse_from_str(&next_time, "%H:%M:%S").unwrap();
    let mut next_days = [2,3,4,5,6,7];
    next_days.sort();
     
    let now = NaiveDate::from_ymd(2022, 10, 30).and_hms(4, 01, 01).into();
    let next = NaiveDate::from_ymd(2022, 11, 1).and_hms(4, 00, 00);
    assert_eq!(next, next_day_in_seq(&next_days, &naive_time, &DateTime::from_utc(now, Utc)));
    }

    #[test]
    fn test_next_day_in_seq_skip_2() {
    let next_time = String::from("04:00:00");
    let naive_time = NaiveTime::parse_from_str(&next_time, "%H:%M:%S").unwrap();
    let mut next_days = [3,4,5,6,7];
    next_days.sort();
     
    let now = NaiveDate::from_ymd(2022, 10, 30).and_hms(4, 01, 01).into();
    let next = NaiveDate::from_ymd(2022, 11, 2).and_hms(4, 00, 00);
    assert_eq!(next, next_day_in_seq(&next_days, &naive_time, &DateTime::from_utc(now, Utc)));
    }

    #[test]
    fn test_next_day_in_seq_skip_time() {
    let next_time = String::from("05:30:00");
    let naive_time = NaiveTime::parse_from_str(&next_time, "%H:%M:%S").unwrap();
    let mut next_days = [2,3,4,5,6,7];
    next_days.sort();
     
    let now = NaiveDate::from_ymd(2022, 10, 30).and_hms(4, 01, 01).into();
    let next = NaiveDate::from_ymd(2022, 11, 1).and_hms(5, 30, 00);
    assert_eq!(next, next_day_in_seq(&next_days, &naive_time, &DateTime::from_utc(now, Utc)));
    }
}