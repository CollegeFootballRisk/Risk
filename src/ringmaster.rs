/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

#![feature(extract_if)]
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate serde_derive;
extern crate rand;
extern crate rand_chacha;
pub mod optional;
pub mod schema;
pub mod structs;

use chrono::{DateTime, Datelike, Duration, NaiveDateTime, NaiveTime, Timelike, Utc};
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::sql_query;
use rand::prelude::*;
use rand_chacha::ChaCha12Rng;
use schema::regions;
use schema::teams;
use schema::territories;
use std::collections::{BTreeMap, HashSet};

const ALT_CUTOFF: i32 = 75;
const AON_END: i32 = 48;
const AON_START: i32 = 4;
const RESPAWN_ENABLED: bool = true;
const MAX_RESPAWN_COUNT: i32 = 1;
const MAIN_SUBMAP_ID: i32 = 0;
const SECONDARY_SUBMAP_ID: i32 = 1;

use structs::{
    Bar, PlayerMoves, Stats, TerritoryOwners, TerritoryOwnersInsert, TerritoryStats, TurnInfo,
    Victor,
};

pub trait Unique<T> {
    fn unique(&mut self) -> Self;
}

impl<T> Unique<T> for Vec<T>
where
    T: Clone + Eq + std::hash::Hash,
{
    fn unique(self: &mut Vec<T>) -> Self {
        let mut seen: HashSet<T> = HashSet::new();
        self.retain(|item| seen.insert(item.clone()));
        return self.to_vec();
    }
}

#[must_use]
pub fn establish_connection() -> PgConnection {
    // Rocket figment gives us the information we need, then we discard it
    let database_url: String = rocket::build()
        .figment()
        .extract_inner("databases.postgres_global.url")
        .expect("Database not set in configuration.");
    PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {database_url}"))
}

fn get_teams(territory_players: Vec<PlayerMoves>) -> Vec<i32> {
    let mut teams = territory_players
        .iter()
        .filter(|mover| mover.alt_score < ALT_CUTOFF)
        .map(|x| x.team)
        .collect::<Vec<i32>>();
    teams.sort_unstable();
    teams.dedup();
    teams
}

/// Based on random number `lottery`, return the ID of the MVP user
fn determine_victor(lottery: f64, map: BTreeMap<i32, Victor>) -> i32 {
    // This is the incrementing value that we will add each team's power to
    let mut victorsum = 0_f64;
    // This will be what we return, the id of the victorious team.
    let mut victor = 0;
    for (key, val) in &map {
        // Add to the sum
        victorsum += val.power;

        // Is the sum greater than the random number?
        if lottery - victorsum < 0_f64 {
            // If so, we have our winner! Go!
            victor = *key;
            break;
        }
    }
    victor
}

// Returns MVP by selecting at random from the team that won
fn get_mvp(mut territory_players: Vec<PlayerMoves>, test: bool) -> Option<PlayerMoves> {
    territory_players.retain(|x| x.alt_score < ALT_CUTOFF && x.power > 0.0);
    let mut seed = if test {
        ChaCha12Rng::seed_from_u64(55)
    } else {
        ChaCha12Rng::from_entropy()
    };
    let rng = match territory_players.len() {
        // We eliminated everyone :(
        0 => return None,
        // In the case there's 1 player, they're it!
        1 => 0,
        // Else, generate the #
        _ => seed.gen_range(0..territory_players.len()),
    };
    // Return the MVP
    Some(territory_players.remove(rng))
}

// Shuffles a list of team IDs
// ```rs
//    let mut teams = vec![1,2,3,4,5];
//    shuffle_teams(&mut teams);
//    // teams is now shuffled
//  ```
fn shuffle_array<T>(items: &mut Vec<T>) {
    let mut rng = thread_rng();

    items.shuffle(&mut rng);
}

// Gathers a pool of available reassignable territories
// (i.e. a team owns multiple territories)
fn gather_reprocessable_territories(
    territories: Vec<TerritoryOwnersInsert>,
) -> (Vec<TerritoryOwnersInsert>, Vec<TerritoryOwnersInsert>) {
    // We'll populate this first with NCAA's territories, then randomly the pre-owned territories
    let mut reprocessable: Vec<TerritoryOwnersInsert> = Vec::new();
    // This holds pre-owned territories
    let mut reprocessable_staging: Vec<TerritoryOwnersInsert> = Vec::new();
    let mut unreprocessable: Vec<TerritoryOwnersInsert> = Vec::new();
    let mut teams_with_prior: Vec<i32> = vec![];
    for territory in territories {
        // If territory's owner owns multiple territories or is NCAA, assign to reprocessable
        if territory.owner_id == 0 {
            reprocessable.push(territory);
        } else if teams_with_prior.contains(&territory.owner_id) {
            reprocessable_staging.push(territory);
        } else {
            // Otherwise, assign to unreprocessable
            teams_with_prior.push(territory.owner_id);
            unreprocessable.push(territory);
        }
    }
    // Shuffle pre-owned territories
    shuffle_array(&mut reprocessable_staging);
    // Appends reprocessible_staging to reprocessable
    reprocessable.append(&mut reprocessable_staging);
    (unreprocessable, reprocessable)
}

// Returns teams which are eligible for respawn
fn get_respawn_elgibile_team_ids(
    team_ids: &Vec<i32>,
    conn: &mut PgConnection,
) -> Result<Vec<i32>, diesel::result::Error> {
    teams::table
        .select(teams::id)
        .filter(teams::id.eq_any(team_ids))
        .filter(teams::respawn_count.lt(MAX_RESPAWN_COUNT))
        .load::<i32>(conn)
}

// Returns teams which are eligible for respawn
fn add_one_to_team_respawns(
    team_ids: &Vec<i32>,
    conn: &mut PgConnection,
) -> Result<usize, diesel::result::Error> {
    diesel::update(teams::table.filter(teams::id.eq_any(team_ids)))
        .set(teams::respawn_count.eq(teams::respawn_count + 1))
        .returning(teams::id)
        .execute(conn)
}

// Returns territory ids which are on the respawn map
// territory_id
// Submaps are the respawn map(s)
fn get_territories_by_submaps(
    submaps: &Vec<i32>,
    conn: &mut PgConnection,
) -> Result<Vec<i32>, diesel::result::Error> {
    territories::table
        .inner_join(regions::table)
        .select(territories::id)
        .filter(regions::submap.eq_any(submaps))
        .load::<i32>(conn)
}

fn reassign_processed_territories(
    eligible_teams: &Vec<i32>,
    territory_pool: &mut Vec<TerritoryOwnersInsert>,
    secondary_stats: &mut BTreeMap<i32, Stats>,
    secondary_territory_stats: &mut Vec<TerritoryStats>,
    turn_id: i32,
) -> Vec<i32> {
    let mut saved_teams: Vec<i32> = vec![];
    let mut territory_pointer = 0;
    for team in eligible_teams {
        if territory_pool.len() > territory_pointer {
            // Remove territory from existing owner's stats
            secondary_stats
                .entry(territory_pool[territory_pointer].owner_id)
                .or_insert_with(|| {
                    Stats::new(turn_id + 1, territory_pool[territory_pointer].owner_id)
                })
                .territorycount -= 1;
            // Update the territory's owner
            territory_pool[territory_pointer].owner_id = *team;
            // Mark as a respawn
            territory_pool[territory_pointer].is_respawn = true;
            // Update respawned team's stats
            secondary_stats
                .entry(*team)
                .or_insert_with(|| Stats::new(turn_id + 1, *team))
                .territorycount += 1;
            // Update territory stats to include this new team
            secondary_territory_stats.push(TerritoryStats {
                team: *team,
                turn_id: turn_id,
                territory: territory_pool[territory_pointer].territory_id,
                ..TerritoryStats::default()
            });
            // Note: we leave the MVP in tact
            territory_pointer += 1;
            saved_teams.push(*team);
        }
    }
    saved_teams
}

/// Go territory by territory and determine new owner, MVP, and statistics
/// This is the portion of the code that determines who wins a territory and
/// who will be MVP of that territory. You can consider it 'RNGESUS,' if you please.
/// Mautamu has added lots of documentation here to understand what's going on.
/// Inputs:
/// - territories: Vec<TerritoryOwners>: These are the _current_ territory owner information
///     e.g. territory id, owner id (current team who owns the territory), turn id (the current turn), and mvp (who won it)
/// - players: Vec<PlayerMoves>: These are the Moves made by the players today that we need to process.
///     e.g. user's id, turn id, territory id, whether they're mvp (it comes in as false but we later tell the DB to set it to true if they're the MVP)
///     the power, multiplier, and weight of the user, by the relationship power = multiplier * weight where weight is a function of starcount (see /src/model/auth/route.rs).
/// Outputs:
/// - Vec<TerritoryOwnersInsert>: The new territory ownership for tomorrow.
/// - Vec<PlayerMoves>: The moves, with MVPs populated
/// - BTreeMap<i32, Stats>: the statistics of the turn for each team
/// - Vec<TerritoryStats>: the statistics of the turn for each territory
fn process_territories(
    territories: Vec<TerritoryOwners>,
    mut players: Vec<PlayerMoves>,
    seed: &mut ChaCha12Rng,
    test: bool,
    must_vacate: Vec<i32>,
) -> (
    Vec<TerritoryOwnersInsert>,
    Vec<PlayerMoves>,
    BTreeMap<i32, Stats>,
    Vec<TerritoryStats>,
) {
    // We log this to STDOUT so we can debug problems
    dbg!("process_territories");
    dbg!(territories.len());

    // We create empty arrays for the outputs
    let mut new_owners: Vec<TerritoryOwnersInsert> = Vec::new();
    let mut mvps: Vec<PlayerMoves> = Vec::new();
    let mut stats: BTreeMap<i32, Stats> = BTreeMap::new();
    let mut territory_stats: Vec<TerritoryStats> = Vec::new();

    // If this is for the respawn map and a team has moved to main map, then we vacate
    // their territories; we do this by discarding their moves and on territories w/ 0 moves
    // reassigning to NCAA (id 0)
    if must_vacate.len() > 0 {
        players.retain(|x| !must_vacate.contains(&x.team));
    }

    // Now we go over each territory that was owned previously.
    // If a territory wasn't 'owned' yesterday, we won't see it.
    // But given proper starting DB conditions that's not an issue.
    for territory in territories {
        // Again, for debugging
        dbg!(&territory.territory_id);

        // We collect all the players that placed a move on this territory
        let territory_players = players
            .extract_if(|player| {
                player.territory == territory.territory_id && player.alt_score < ALT_CUTOFF
            })
            .collect::<Vec<_>>();

        // Again, for debugging
        dbg!(&territory_players.len());

        // This function returns the teams that attacked/defended this territory
        // It does so by collecting the team id from all players and then removing dupes.
        let teams = get_teams(territory_players.clone());

        // Here we split into different logic depending on how many teams attacked a territory.
        // If nobody made a move on the territory (e.g. it's surrounded by territories also owned by the same owner)
        // then we keep it the same.
        // If only one team made a move on the territory (and they put some power forth, i.e. no alts)
        // then we assign that team the territory.
        // If more than one team made a move on a territory (and they put some power forth, i.e. no alts)
        // then we need to use the RNG to determine the winner.
        match teams.len() {
            // This is the "no teams attacked" case, so keep the owner the same.
            0 => {
                dbg!("Zero Team");

                // In the case that respawn = true and the owning team
                let territory_owner_id = if must_vacate.len() > 0 {
                    0
                } else {
                    territory.owner_id
                };

                // Push the new territory owner.
                new_owners.push(TerritoryOwnersInsert::new(
                    &territory,
                    territory_owner_id,
                    None,
                    None,
                    false,
                ));

                // add team territory count to stats (but no MVPs as nobody made a move here)
                stats
                    .entry(territory_owner_id)
                    .or_insert_with(|| Stats::new(territory.turn_id + 1, territory_owner_id))
                    .territorycount += 1;
                territory_stats.push(TerritoryStats {
                    team: territory_owner_id,
                    turn_id: territory.turn_id,
                    territory: territory.territory_id,
                    ..TerritoryStats::default()
                });
                continue; // next territory
            }
            // This is the case where only one team acted on a territory.
            1 => {
                // Due to All-or-nothing and alts, we don't get to just assume that this team gets it
                // So let's check if there's any power available from the team.
                if territory_players
                    .iter()
                    .filter(|mover| mover.alt_score < ALT_CUTOFF)
                    .map(|mover| mover.power)
                    .sum::<f64>()
                    == 0.0
                {
                    // There is no power from this team, this is the same case as if there is no teams, next
                    dbg!("Team has no power");
                    new_owners.push(TerritoryOwnersInsert::new(
                        &territory,
                        territory.owner_id,
                        None,
                        None,
                        false,
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

                    let team_star_breakdown = get_team_star_breakdown(&territory_players);
                    if team_star_breakdown.iter().sum::<i32>() > 0 {
                        territory_stats.push(TerritoryStats {
                            team: territory_players[0].team,
                            turn_id: territory.turn_id,
                            ones: team_star_breakdown[0],
                            twos: team_star_breakdown[1],
                            threes: team_star_breakdown[2],
                            fours: team_star_breakdown[3],
                            fives: team_star_breakdown[4],
                            teampower: 0.0,
                            chance: 0.0,
                            territory: territory.territory_id,
                            territory_power: 0.0,
                        });
                    }
                    // We don't know if it was an alt or not so this helps us out
                    handle_team_stats(&mut stats, territory_players);
                    continue; // next territory
                }

                // There was some power from the team! Let's give them the territory.
                dbg!("One Team");

                // We select an MVP
                let mvp = get_mvp(territory_players.clone(), test);
                // We push the mvps onto the MVP docket from earlier.
                let mvp_id = match mvp {
                    None => None,
                    Some(mvp_i) => {
                        // Don't forget to record the MVP!
                        mvps.push(mvp_i.clone());

                        // Return ID
                        Some(mvp_i.user_id)
                    }
                };

                // We push the new territory owner into the owners docket.
                new_owners.push(TerritoryOwnersInsert::new(
                    &territory, teams[0], None, mvp_id, false,
                ));

                // And finally we alter the statistics for the team that won
                stats
                    .entry(teams[0])
                    .or_insert_with(|| Stats::new(territory.turn_id + 1, teams[0]))
                    .territorycount += 1;

                // Imagine a team quit the game, we want to show their stats on the leaderboard.
                // So we create a stats entry for them with 0 territories. OR they abandoned this
                // territory so we'll just add 0 territories while we're at it.
                stats
                    .entry(territory.owner_id)
                    .or_insert_with(|| Stats::new(territory.turn_id + 1, territory.owner_id))
                    .territorycount += 0;

                // You may have noticed we only updated the territory count previously.
                // But team stats includes mvps and power, et al. We calculate that now.
                // add team stats
                handle_team_stats(&mut stats, territory_players.clone());
                // This team might be dead/abandoned this territory, so we push to the odds table
                if teams[0] != territory.owner_id {
                    territory_stats.push(TerritoryStats {
                        team: territory.owner_id,
                        turn_id: territory.turn_id,
                        territory: territory.territory_id,
                        territory_power: territory_players
                            .iter()
                            .filter(|mover| mover.alt_score < ALT_CUTOFF)
                            .map(|mover| mover.power)
                            .sum::<f64>(),
                        chance: 0.00,
                        ..TerritoryStats::default()
                    });
                }

                let team_star_breakdown = get_team_star_breakdown(&territory_players);

                // Finally we push the statistics for the territory
                territory_stats.push(TerritoryStats {
                    team: teams[0],
                    turn_id: territory.turn_id,
                    ones: team_star_breakdown[0],
                    twos: team_star_breakdown[1],
                    threes: team_star_breakdown[2],
                    fours: team_star_breakdown[3],
                    fives: team_star_breakdown[4],
                    teampower: territory_players
                        .iter()
                        .filter(|player| player.alt_score < ALT_CUTOFF)
                        .map(|mover| mover.power)
                        .sum::<f64>(),
                    chance: 1.00,
                    territory: territory.territory_id,
                    territory_power: territory_players
                        .iter()
                        .filter(|player| player.alt_score < ALT_CUTOFF)
                        .map(|mover| mover.power)
                        .sum::<f64>(),
                });
                continue; // The territory is processed, on to the next one (if there's only one team that is)!
            }
            // In Rust, `_` is a catchall for everything not in the match already.
            // In this case, it means ALL territories which have > 1 team.
            _ => {
                // Again, for debugging.
                dbg!(&teams);

                // Due to All-or-nothing and alts, we don't get to just assume that this team gets it
                // So let's check if there's any power available from the team.
                if territory_players
                    .iter()
                    .filter(|player| player.alt_score < ALT_CUTOFF)
                    .map(|mover| mover.power)
                    .sum::<f64>()
                    == 0.0
                {
                    // No team has power, this is the same case as if there is no teams, next
                    dbg!("Teams have no power");

                    // Push the same owner as previously
                    new_owners.push(TerritoryOwnersInsert::new(
                        &territory,
                        territory.owner_id,
                        None,
                        None,
                        false,
                    ));

                    // add team territory count to stats
                    stats
                        .entry(territory.owner_id)
                        .or_insert_with(|| Stats::new(territory.turn_id + 1, territory.owner_id))
                        .territorycount += 1;

                    // and finally push territory stats
                    territory_stats.push(TerritoryStats {
                        team: territory.owner_id,
                        turn_id: territory.turn_id,
                        territory: territory.territory_id,
                        ..TerritoryStats::default()
                    });

                    // Now we do the same for the other teams in case they might have non-alts.

                    for team in teams {
                        let team_star_breakdown = get_team_star_breakdown(
                            &territory_players
                                .iter()
                                .filter(|p| p.team == team)
                                .cloned()
                                .collect(),
                        );
                        if team_star_breakdown.iter().sum::<i32>() > 0 {
                            territory_stats.push(TerritoryStats {
                                team,
                                turn_id: territory.turn_id,
                                ones: team_star_breakdown[0],
                                twos: team_star_breakdown[1],
                                threes: team_star_breakdown[2],
                                fours: team_star_breakdown[3],
                                fives: team_star_breakdown[4],
                                teampower: 0.0,
                                chance: 0.0,
                                territory: territory.territory_id,
                                territory_power: 0.0,
                            });
                        }
                        // We don't know if it was an alt or not so this helps us out
                        handle_team_stats(
                            &mut stats,
                            territory_players
                                .iter()
                                .filter(|p| p.team == team)
                                .cloned()
                                .collect(),
                        );
                    }

                    continue; // next territory
                }

                // We create this empty BTreeMap for collecting team information
                let mut map = BTreeMap::new();

                // We create a stats entry for the owner of the territory, just in case they abandoned it
                stats
                    .entry(territory.owner_id)
                    .or_insert_with(|| Stats::new(territory.turn_id + 1, territory.owner_id))
                    .territorycount += 0;

                // For each team, insert the team into the BTreeMap we made earlier
                // in this case, Victor is a set of stats metrics (# by star, overall power)
                for team in teams {
                    map.insert(team, Victor::default()); // stars, power, ones, twos, threes, fours, fives
                }

                // Now we populate the map with the power
                for player in &territory_players {
                    //dbg!(player.id, player.team);
                    if player.alt_score >= ALT_CUTOFF {
                        continue;
                    }

                    // The .power() and .stars() calls add the power to the team's pile
                    // and adds +1 to the # of players with the current player's starcount (1,2,3,4,5)
                    map.get_mut(&player.team)
                        .unwrap()
                        .power(player.power)
                        .stars(player.stars);
                }

                // We now calculate the total power that was expended by ALL teams on the territory.
                let totalpower: f64 = map.values().map(|x| (x.power)).sum();

                // We now generate the random number from 0 to the total power on the territory.
                // We pass this in so we can run tests
                let lottery = seed.gen_range(0_f64..totalpower);

                // We determine the victor, this function goes team by team until the sum of the power of teams
                // it has reviewed is greater than `lottery` the random number we created earlier.
                // It returns the id of the team that wins.
                let victor = determine_victor(lottery, map.clone());

                // We collect the players that are on the winning team for MVPing.
                let territory_victors = territory_players
                    .clone()
                    .extract_if(|player| player.team == victor && player.alt_score < ALT_CUTOFF)
                    .collect::<Vec<_>>();

                // We now determine the MVP from the players on the winning team.
                let mvp = get_mvp(territory_victors, test);
                let mvp_id = match mvp {
                    None => None,
                    Some(mvp_i) => {
                        // Don't forget to record the MVP!
                        mvps.push(mvp_i.clone());

                        // Return ID
                        Some(mvp_i.user_id)
                    }
                };

                // We push the new owner
                new_owners.push(TerritoryOwnersInsert::new(
                    &territory,
                    victor,
                    Some(lottery),
                    mvp_id,
                    false,
                ));

                // We generate the team statistics for the victor
                stats
                    .entry(victor)
                    .or_insert_with(|| Stats::new(territory.turn_id + 1, victor))
                    .territorycount += 1;

                // We now calculate total power for the territory for territory statistics..
                handle_team_stats(&mut stats, territory_players);

                // And we then push the territory statistics.
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
                        chance: val.power / totalpower,
                        territory: territory.territory_id,
                        territory_power: totalpower,
                    });
                }

                // And as a final check, if the previous owner isn't in the stats
                // but did move on the territory, we update their statistics.
                if !map.contains_key(&territory.owner_id) {
                    territory_stats.push(TerritoryStats {
                        team: territory.owner_id,
                        turn_id: territory.turn_id,
                        territory: territory.territory_id,
                        territory_power: totalpower,
                        chance: 0.00,
                        ..TerritoryStats::default()
                    });
                }
            }
        }
    }

    // Sort if this is a test
    if test {
        new_owners.sort_by_key(|p| p.territory_id);
        mvps.sort_by_key(|p| p.user_id);
        territory_stats.sort_by_key(|p| p.team);
    }

    // Finally, we return everything we just calculated.
    (new_owners, mvps, stats, territory_stats)
}

fn handle_team_stats(stats: &mut BTreeMap<i32, Stats>, territory_players: Vec<PlayerMoves>) {
    for i in territory_players {
        if i.alt_score >= ALT_CUTOFF {
            continue;
        }
        let starpower = if i.power == 0.0 {
            0.0
        } else {
            i.power / i.multiplier
        };
        stats
            .entry(i.team)
            .or_insert_with(|| Stats::new(i.turn_id + 1, i.team))
            .starpower(starpower)
            .effectivepower(i.power.round())
            .add_player_or_merc(i.merc)
            .stars(i.stars);
    }
}

fn get_team_star_breakdown(territory_player: &Vec<PlayerMoves>) -> [i32; 5] {
    let mut output = [0, 0, 0, 0, 0];

    for pm in territory_player {
        if pm.alt_score >= ALT_CUTOFF {
            continue;
        }
        match pm.stars {
            1 => output[0] += 1,
            2 => output[1] += 1,
            3 => output[2] += 1,
            4 => output[3] += 1,
            5 => output[4] += 1,
            _ => output[0] += 1,
        }
    }

    output
}

/// Updates the statistics for all users after the roll
fn user_update(
    turninfoblock: &TurnInfo,
    conn: &mut PgConnection,
) -> Result<Vec<Bar>, diesel::result::Error> {
    let query = format!(
        "SELECT do_user_update({},{})",
        &turninfoblock.id.to_string(),
        &turninfoblock.season.to_string(),
    );
    sql_query(query).load(conn)
}

/// Updates chaos bridges randomly
#[cfg(feature = "chaos")]
fn chaos_update(
    territories: &[TerritoryOwnersInsert],
    turn_id_n: i32,
    settings: &rocket::figment::Figment,
    conn: &mut PgConnection,
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
    #[diesel(table_name = "territory_adjacency")]
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

fn next_roll(settings: &rocket::figment::Figment) -> Option<NaiveDateTime> {
    // Calculate new starttime
    let next_time = settings
        .extract_inner::<String>("risk.time")
        .unwrap_or_else(|_| String::from("04:00:00"));
    let naive_time = NaiveTime::parse_from_str(&next_time, "%H:%M:%S").unwrap();
    let next_days = settings
        .extract_inner::<Vec<i64>>("risk.days")
        .unwrap_or_else(|_| vec![1, 2, 3, 4, 5, 6, 7]);
    next_day_in_seq(&next_days, &naive_time, &Utc::now())
}

// Function assumes that we're after today's roll
fn next_day_in_seq(
    next_days: &[i64],
    next_time: &NaiveTime,
    now: &DateTime<Utc>,
) -> Option<NaiveDateTime> {
    let curr_day: i64 = i64::from(now.weekday().number_from_monday());
    let index: i64 = if next_days.is_empty() {
        return None;
    } else if let Some(next) = next_days.iter().filter(|&x| *x > curr_day).min() {
        *next - curr_day
    } else {
        let min = next_days.iter().min().unwrap_or(&0);
        7 - (min - curr_day).abs()
    };
    (*now + Duration::try_days(index).expect("adding index of days should work"))
        .date_naive()
        .and_hms_opt(next_time.hour(), next_time.minute(), next_time.second())
}

fn runtime() -> Result<(), diesel::result::Error> {
    let rocket = rocket::build();
    let settings = rocket.figment();
    // Connect to the Postgres DB
    let mut conn: PgConnection = establish_connection();
    // Get the active turn
    // start_time_now then sets the start time to the current time.
    let mut turninfoblock = TurnInfo::get_latest(&mut conn)?;
    turninfoblock.start_time_now();
    // Prevent new moves from being submitted
    turninfoblock.lock(&mut conn)?;
    //dbg!(&turninfoblock.season, &turninfoblock.day);
    // Now we go get all player moves for the current day
    let players = PlayerMoves::load(&turninfoblock.id, &mut conn)?;
    // If there are no moves to load, we'll exit as something's not right.
    // TODO: Return Err, not Ok
    if players.is_empty() {
        return Ok(());
    }

    // If Respawn is enabled, select two pools of territories
    let (territories, secondary_territories, primary_players, secondary_players) =
        if RESPAWN_ENABLED {
            let territory_id_filter_main: Vec<i32> =
                get_territories_by_submaps(&vec![MAIN_SUBMAP_ID], &mut conn)?;
            let territory_id_filter_secondary: Vec<i32> =
                get_territories_by_submaps(&vec![SECONDARY_SUBMAP_ID], &mut conn)?;
            let territories: Vec<TerritoryOwners> = TerritoryOwners::load(
                &turninfoblock.id,
                Some(&territory_id_filter_main),
                &mut conn,
            )?;
            let secondary_territories = TerritoryOwners::load(
                &turninfoblock.id,
                Some(&territory_id_filter_secondary),
                &mut conn,
            )?;

            let (primary_players, secondary_players) = players
                .into_iter()
                .partition(|v| territory_id_filter_main.contains(&v.territory));

            (
                territories,
                Some(secondary_territories),
                primary_players,
                Some(secondary_players),
            )
        } else {
            // And a list of all territories, and their current owners:
            let territories = TerritoryOwners::load(&turninfoblock.id, None, &mut conn)?;
            (territories, None, players, None)
        };

    // If Respawn is enabled, get only those territories for which the territory is
    // on the main map
    //let move_ids = players.iter().map(|x| x.id).collect::<Vec<i32>>();
    // We pass in an entropy-driven randomy number, since we're not testing
    let (mut owners, mut mvps, mut stats, mut territory_stats) = process_territories(
        territories,
        primary_players,
        &mut ChaCha12Rng::from_entropy(),
        false,
        Vec::new(),
    );
    // If Respawn is enabled, deduce which teams died
    if RESPAWN_ENABLED {
        let alive_teams = owners
            .iter()
            .map(|v| v.owner_id)
            .collect::<Vec<i32>>()
            .unique();
        let eliminated_teams = owners
            .iter()
            .map(|v| v.previous_owner_id)
            .collect::<Vec<i32>>()
            .unique()
            .iter()
            .filter(|v| !alive_teams.contains(*v))
            .map(|v| v.clone())
            .collect();
        // Now that we know which teams died, query to see which are eligible for respawn
        let mut eligible_eliminated_teams =
            get_respawn_elgibile_team_ids(&eliminated_teams, &mut conn)?;

        // Run process_territories on secondary_territories
        // If any teams alive on the main map exist, we then vacate their territories
        let (
            mut secondary_owners,
            mut secondary_mvps,
            mut secondary_stats,
            mut secondary_territory_stats,
        ) = process_territories(
            secondary_territories.expect("Secondary territories not defined, bailing!"),
            secondary_players.expect("Secondary players was neither a populated nor empty vec?"),
            &mut ChaCha12Rng::from_entropy(),
            false,
            alive_teams,
        );

        if eligible_eliminated_teams.len() > 0 {
            // Shuffle the pool of eliminated teams
            shuffle_array(&mut eligible_eliminated_teams);
            // Gather a pool of available reassignable territories
            let (unreprocessable, mut reprocessable) =
                gather_reprocessable_territories(secondary_owners.clone());

            secondary_owners = unreprocessable;
            // Assign eliminated teams territories from the reprocessable pool, if eligible
            let saved_teams = reassign_processed_territories(
                &eligible_eliminated_teams,
                &mut reprocessable,
                &mut secondary_stats,
                &mut secondary_territory_stats,
                turninfoblock.id,
            );
            // Update the statistics and whatnot
            assert_eq!(
                add_one_to_team_respawns(&saved_teams, &mut conn)?,
                saved_teams.len()
            );
        }

        // Merge all the data
        owners.append(&mut secondary_owners);
        mvps.append(&mut secondary_mvps);
        stats.append(&mut secondary_stats);
        territory_stats.append(&mut secondary_territory_stats);
    }

    TerritoryStats::insert(territory_stats, &mut conn)?;
    Stats::insert(stats, turninfoblock.id, &mut conn)?;
    let territory_insert = TerritoryOwnersInsert::insert(&owners, &mut conn)?;
    dbg!(territory_insert);
    let playermoves = PlayerMoves::mvps(mvps, &mut conn)?;
    dbg!(playermoves);

    // This calls an SQL function that updates each user's statistics
    // Not ideal, TODO: we ought to implement this in Rust.
    let userupdate = user_update(&turninfoblock, &mut conn);
    match userupdate {
        Ok(ok) => println!("Users updated successfully {}", ok[0].do_user_update),
        Err(e) => println!("Failed to update users: {e:?}"),
    }
    turninfoblock.rollendtime = Some(Utc::now().naive_utc());
    turninfoblock.complete = Some(true);
    turninfoblock.active = Some(false);
    match TurnInfo::update_or_insert(&turninfoblock, &mut conn) {
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
        next_roll(settings),
        &mut conn,
    ) {
        Ok(_ok) => {
            println!("Create new turn succeeded")
        }
        Err(e) => println!("Failed to make new turn {e:?}"),
    }

    #[cfg(feature = "risk_image")]
    optional::image::make_image(&owners, &mut conn);

    #[cfg(feature = "chaos")]
    {
        match chaos_update(&owners, turninfoblock.id + 1, settings, &mut conn) {
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
    println!("Elapsed: {elapsed:.2?}");
    println!("Start Time: {now:.2?}");
    println!("End Time: {end:.2?}");
    std::process::exit(match state {
        Ok(_) => 0,
        Err(err) => {
            eprintln!("error: {err:?}");
            1
        }
    });
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    use chrono::{NaiveDate, NaiveTime};

    #[test]
    fn test_next_day_in_seq() {
        let next_time = String::from("04:00:00");
        let naive_time = NaiveTime::parse_from_str(&next_time, "%H:%M:%S").unwrap();
        let mut next_days = [1, 2, 3, 4, 5, 6, 7];
        next_days.sort_unstable();

        let now = NaiveDate::from_ymd_opt(2022, 10, 30)
            .unwrap()
            .and_hms_opt(4, 1, 1)
            .unwrap();
        let next = NaiveDate::from_ymd_opt(2022, 10, 31)
            .unwrap()
            .and_hms_opt(4, 0, 0)
            .unwrap();
        assert_eq!(
            Some(next),
            next_day_in_seq(
                &next_days,
                &naive_time,
                &DateTime::from_naive_utc_and_offset(now, Utc)
            )
        );
    }

    #[test]
    fn test_next_day_in_seq_empty() {
        let next_time = String::from("04:00:00");
        let naive_time = NaiveTime::parse_from_str(&next_time, "%H:%M:%S").unwrap();
        let mut next_days = [];
        next_days.sort_unstable();

        let now = NaiveDate::from_ymd_opt(2022, 10, 30)
            .unwrap()
            .and_hms_opt(4, 1, 1)
            .unwrap();
        assert_eq!(
            None,
            next_day_in_seq(
                &next_days,
                &naive_time,
                &DateTime::from_naive_utc_and_offset(now, Utc)
            )
        );
    }

    #[test]
    fn test_next_day_in_seq_skip() {
        let next_time = String::from("04:00:00");
        let naive_time = NaiveTime::parse_from_str(&next_time, "%H:%M:%S").unwrap();
        let mut next_days = [2, 3, 4, 5, 6, 7];
        next_days.sort_unstable();

        let now = NaiveDate::from_ymd_opt(2022, 10, 30)
            .unwrap()
            .and_hms_opt(4, 1, 1)
            .unwrap();
        let next = NaiveDate::from_ymd_opt(2022, 11, 1)
            .unwrap()
            .and_hms_opt(4, 0, 0)
            .unwrap();
        assert_eq!(
            Some(next),
            next_day_in_seq(
                &next_days,
                &naive_time,
                &DateTime::from_naive_utc_and_offset(now, Utc)
            )
        );
    }

    #[test]
    fn test_gather_reprocessable_territories() {
        let territories: Vec<TerritoryOwnersInsert> = vec![
            TerritoryOwnersInsert {
                territory_id: 1,
                owner_id: 2,
                ..Default::default()
            },
            TerritoryOwnersInsert {
                territory_id: 2,
                owner_id: 2,
                ..Default::default()
            },
            TerritoryOwnersInsert {
                territory_id: 3,
                owner_id: 0,
                ..Default::default()
            },
            TerritoryOwnersInsert {
                territory_id: 4,
                owner_id: 3,
                ..Default::default()
            },
            TerritoryOwnersInsert {
                territory_id: 5,
                owner_id: 0,
                ..Default::default()
            },
        ];
        let (unreprocessable, reprocessable) = gather_reprocessable_territories(territories);
        assert_eq!(
            unreprocessable
                .iter()
                .map(|x| x.territory_id)
                .collect::<Vec<i32>>(),
            vec![1, 4]
        );
        // Make sure owner_id=0 are placed first
        assert_eq!(
            reprocessable
                .iter()
                .map(|x| x.territory_id)
                .collect::<Vec<i32>>(),
            vec![3, 5, 2]
        );
    }

    #[test]
    fn test_next_day_in_seq_skip_2() {
        let next_time = String::from("04:00:00");
        let naive_time = NaiveTime::parse_from_str(&next_time, "%H:%M:%S").unwrap();
        let mut next_days = [3, 4, 5, 6, 7];
        next_days.sort_unstable();

        let now = NaiveDate::from_ymd_opt(2022, 10, 30)
            .unwrap()
            .and_hms_opt(4, 1, 1)
            .unwrap();
        let next = NaiveDate::from_ymd_opt(2022, 11, 2)
            .unwrap()
            .and_hms_opt(4, 0, 0)
            .unwrap();
        assert_eq!(
            Some(next),
            next_day_in_seq(
                &next_days,
                &naive_time,
                &DateTime::from_naive_utc_and_offset(now, Utc)
            )
        );
    }

    #[test]
    fn test_season_2_params() {
        let next_time = String::from("03:30:00");
        let naive_time = NaiveTime::parse_from_str(&next_time, "%H:%M:%S").unwrap();
        let mut next_days = [2, 3, 4, 5, 6, 7];
        next_days.sort_unstable();

        let now = NaiveDate::from_ymd_opt(2022, 11, 26)
            .unwrap()
            .and_hms_opt(3, 30, 1)
            .unwrap();
        let next = NaiveDate::from_ymd_opt(2022, 11, 27)
            .unwrap()
            .and_hms_opt(3, 30, 0)
            .unwrap();
        assert_eq!(
            Some(next),
            next_day_in_seq(
                &next_days,
                &naive_time,
                &DateTime::from_naive_utc_and_offset(now, Utc)
            )
        );
    }

    #[test]
    fn test_each_day() {
        for i in 0..7 {
            let next_time = String::from("03:30:00");
            let naive_time = NaiveTime::parse_from_str(&next_time, "%H:%M:%S").unwrap();
            let mut next_days = [1, 2, 3, 4, 5, 6, 7];
            next_days.sort_unstable();

            let now = NaiveDate::from_ymd_opt(2022, 11, 20 + i)
                .unwrap()
                .and_hms_opt(3, 30, 1)
                .unwrap();
            let next = NaiveDate::from_ymd_opt(2022, 11, 21 + i)
                .unwrap()
                .and_hms_opt(3, 30, 0)
                .unwrap();
            assert_eq!(
                Some(next),
                next_day_in_seq(
                    &next_days,
                    &naive_time,
                    &DateTime::from_naive_utc_and_offset(now, Utc)
                )
            );
        }
    }

    #[test]
    fn test_each_day_with_skip() {
        for i in 0..8 {
            let next_time = String::from("03:30:00");
            let naive_time = NaiveTime::parse_from_str(&next_time, "%H:%M:%S").unwrap();
            let mut next_days = [2, 3, 4, 5, 6, 7];
            next_days.sort_unstable();

            let now = NaiveDate::from_ymd_opt(2022, 11, 20 + i)
                .unwrap()
                .and_hms_opt(3, 30, 1)
                .unwrap();
            let next = if i % 7 == 0 {
                NaiveDate::from_ymd_opt(2022, 11, 21 + i + 1)
                    .unwrap()
                    .and_hms_opt(3, 30, 0)
                    .unwrap()
            } else {
                NaiveDate::from_ymd_opt(2022, 11, 21 + i)
                    .unwrap()
                    .and_hms_opt(3, 30, 0)
                    .unwrap()
            };
            assert_eq!(
                Some(next),
                next_day_in_seq(
                    &next_days,
                    &naive_time,
                    &DateTime::from_naive_utc_and_offset(now, Utc)
                )
            );
        }
    }

    #[test]
    fn test_get_mvp_empty() {
        let playermoves: Vec<PlayerMoves> = Vec::new();
        assert_eq!(None, get_mvp(playermoves, false));
    }

    #[test]
    fn test_get_mvp_alt() {
        let playermoves: Vec<PlayerMoves> = vec![PlayerMoves {
            id: 32,
            user_id: 32,
            turn_id: 32,
            territory: 32,
            mvp: false,
            power: 2.0,
            multiplier: 2.0,
            weight: 1.0,
            stars: 2,
            team: 20,
            alt_score: ALT_CUTOFF + 1,
            merc: false,
        }];
        assert_eq!(None, get_mvp(playermoves, false));
    }

    #[test]
    fn test_get_mvp_power() {
        let playermoves: Vec<PlayerMoves> = vec![PlayerMoves {
            id: 32,
            user_id: 32,
            turn_id: 32,
            territory: 32,
            mvp: false,
            power: 0.0,
            multiplier: 0.0,
            weight: 1.0,
            stars: 2,
            team: 20,
            alt_score: 0,
            merc: false,
        }];
        assert_eq!(None, get_mvp(playermoves, false));
    }

    #[test]
    fn test_get_mvp_one() {
        let playermoves: Vec<PlayerMoves> = vec![PlayerMoves {
            id: 32,
            user_id: 32,
            turn_id: 32,
            territory: 32,
            mvp: false,
            power: 10.0,
            multiplier: 10.0,
            weight: 1.0,
            stars: 2,
            team: 20,
            alt_score: 0,
            merc: false,
        }];
        assert_eq!(Some(playermoves[0].clone()), get_mvp(playermoves, false));
    }

    #[test]
    fn test_get_mvp_two() {
        let playermoves: Vec<PlayerMoves> = vec![
            PlayerMoves {
                id: 32,
                user_id: 32,
                turn_id: 32,
                territory: 32,
                mvp: false,
                power: 10.0,
                multiplier: 10.0,
                weight: 1.0,
                stars: 2,
                team: 20,
                alt_score: 0,
                merc: false,
            },
            PlayerMoves {
                id: 34,
                user_id: 34,
                turn_id: 32,
                territory: 32,
                mvp: false,
                power: 15.0,
                multiplier: 15.0,
                weight: 1.0,
                stars: 2,
                team: 20,
                alt_score: 0,
                merc: false,
            },
        ];
        assert_eq!(Some(playermoves[1].clone()), get_mvp(playermoves, true));
    }

    #[test]
    fn test_process_territories_zero() {
        let territories = vec![TerritoryOwners {
            id: 1,
            territory_id: 2,
            owner_id: 3,
            turn_id: 4,
            previous_owner_id: 5,
            random_number: 0.0,
            mvp: None,
            is_respawn: false,
        }];

        let playermoves: Vec<PlayerMoves> = Vec::new();
        let new_owners = vec![TerritoryOwnersInsert::new(
            &territories[0],
            3,
            Some(0.0),
            None,
            false,
        )];
        let mvps: Vec<PlayerMoves> = Vec::new();
        let mut stats: BTreeMap<i32, Stats> = BTreeMap::new();
        stats
            .entry(3)
            .or_insert_with(|| Stats::new(5, 3))
            .territorycount = 1;
        let territory_stats: Vec<TerritoryStats> = vec![TerritoryStats {
            team: 3,
            turn_id: 4,
            territory: 2,
            ..TerritoryStats::default()
        }];

        // Because we are only testing functionality, not actually running ringmaster
        // we pass a known seed for reproducible results.
        assert_eq!(
            (new_owners, mvps, stats, territory_stats),
            process_territories(
                territories,
                playermoves,
                &mut ChaCha12Rng::seed_from_u64(45),
                true,
                Vec::new()
            )
        );
    }

    #[test]
    fn test_process_territories_one_same() {
        let territories = vec![TerritoryOwners {
            id: 1,
            territory_id: 2,
            owner_id: 3,
            turn_id: 4,
            previous_owner_id: 5,
            random_number: 0.0,
            mvp: None,
            is_respawn: false,
        }];

        let playermoves: Vec<PlayerMoves> = vec![PlayerMoves {
            id: 45,
            user_id: 6,
            turn_id: 4,
            territory: 2,
            mvp: false,
            power: 12.0,
            multiplier: 1.0,
            weight: 12.0,
            stars: 5,
            team: 3,
            alt_score: 0,
            merc: false,
        }];
        let new_owners = vec![TerritoryOwnersInsert::new(
            &territories[0],
            3,
            Some(0.0),
            Some(6),
            false,
        )];
        let mvps: Vec<PlayerMoves> = vec![playermoves[0].clone()];
        let mut stats: BTreeMap<i32, Stats> = BTreeMap::new();
        stats
            .entry(3)
            .or_insert_with(|| Stats::new(5, 3))
            .territorycount = 1;
        stats
            .entry(3)
            .or_insert_with(|| Stats::new(5, 3))
            .playercount = 1;
        stats.entry(3).or_insert_with(|| Stats::new(5, 3)).starpower = 12.0;
        stats
            .entry(3)
            .or_insert_with(|| Stats::new(5, 3))
            .efficiency = 0.0;
        stats
            .entry(3)
            .or_insert_with(|| Stats::new(5, 3))
            .effectivepower = 12.0;
        stats.entry(3).or_insert_with(|| Stats::new(5, 3)).fives = 1;
        let territory_stats: Vec<TerritoryStats> = vec![TerritoryStats {
            team: 3,
            turn_id: 4,
            territory: 2,
            territory_power: 12.0,
            chance: 1.0,
            teampower: 12.0,
            fives: 1,
            ..TerritoryStats::default()
        }];

        // Because we are only testing functionality, not actually running ringmaster
        // we pass a known seed for reproducible results.
        assert_eq!(
            (new_owners, mvps, stats, territory_stats),
            process_territories(
                territories,
                playermoves,
                &mut ChaCha12Rng::seed_from_u64(45),
                true,
                Vec::new()
            )
        );
    }

    #[test]
    fn test_process_territories_one_diff() {
        let territories = vec![TerritoryOwners {
            id: 1,
            territory_id: 2,
            owner_id: 6,
            turn_id: 4,
            previous_owner_id: 5,
            random_number: 0.0,
            mvp: None,
            is_respawn: false,
        }];

        let playermoves: Vec<PlayerMoves> = vec![PlayerMoves {
            id: 45,
            user_id: 6,
            turn_id: 4,
            territory: 2,
            mvp: false,
            power: 12.0,
            multiplier: 1.0,
            weight: 12.0,
            stars: 5,
            team: 3,
            alt_score: 0,
            merc: false,
        }];
        let new_owners = vec![TerritoryOwnersInsert::new(
            &territories[0],
            3,
            Some(0.0),
            Some(6),
            false,
        )];
        let mvps: Vec<PlayerMoves> = vec![playermoves[0].clone()];
        let mut stats: BTreeMap<i32, Stats> = BTreeMap::new();
        stats
            .entry(6)
            .or_insert_with(|| Stats::new(5, 6))
            .territorycount = 0;

        stats
            .entry(3)
            .or_insert_with(|| Stats::new(5, 3))
            .territorycount = 1;
        stats
            .entry(3)
            .or_insert_with(|| Stats::new(5, 3))
            .playercount = 1;
        stats.entry(3).or_insert_with(|| Stats::new(5, 3)).starpower = 12.0;
        stats
            .entry(3)
            .or_insert_with(|| Stats::new(5, 3))
            .efficiency = 0.0;
        stats
            .entry(3)
            .or_insert_with(|| Stats::new(5, 3))
            .effectivepower = 12.0;
        stats.entry(3).or_insert_with(|| Stats::new(5, 3)).fives = 1;
        let territory_stats: Vec<TerritoryStats> = vec![
            TerritoryStats {
                team: 3,
                turn_id: 4,
                territory: 2,
                territory_power: 12.0,
                chance: 1.0,
                teampower: 12.0,
                fives: 1,
                ..TerritoryStats::default()
            },
            TerritoryStats {
                team: 6,
                turn_id: 4,
                territory: 2,
                territory_power: 12.0,
                chance: 0.0,
                teampower: 0.0,
                ..TerritoryStats::default()
            },
        ];

        // Because we are only testing functionality, not actually running ringmaster
        // we pass a known seed for reproducible results.
        assert_eq!(
            (new_owners, mvps, stats, territory_stats),
            process_territories(
                territories,
                playermoves,
                &mut ChaCha12Rng::seed_from_u64(45),
                true,
                Vec::new()
            )
        );
    }

    #[test]
    fn test_process_territories_one_diff_powerless() {
        let territories = vec![TerritoryOwners {
            id: 1,
            territory_id: 2,
            owner_id: 6,
            turn_id: 4,
            previous_owner_id: 5,
            random_number: 0.0,
            mvp: None,
            is_respawn: false,
        }];

        let playermoves: Vec<PlayerMoves> = vec![PlayerMoves {
            id: 45,
            user_id: 6,
            turn_id: 4,
            territory: 2,
            mvp: false,
            power: 0.0,
            multiplier: 0.0,
            weight: 12.0,
            stars: 5,
            team: 3,
            alt_score: 0,
            merc: false,
        }];
        let new_owners = vec![TerritoryOwnersInsert::new(
            &territories[0],
            6,
            Some(0.0),
            None,
            false,
        )];
        let mvps: Vec<PlayerMoves> = vec![];
        let mut stats: BTreeMap<i32, Stats> = BTreeMap::new();
        stats
            .entry(3)
            .or_insert_with(|| Stats::new(5, 3))
            .territorycount = 0;
        stats
            .entry(3)
            .or_insert_with(|| Stats::new(5, 3))
            .playercount = 1;
        stats.entry(3).or_insert_with(|| Stats::new(5, 3)).starpower = 0.0;
        stats
            .entry(3)
            .or_insert_with(|| Stats::new(5, 3))
            .efficiency = 0.0;
        stats
            .entry(3)
            .or_insert_with(|| Stats::new(5, 3))
            .effectivepower = 0.0;
        stats
            .entry(6)
            .or_insert_with(|| Stats::new(5, 6))
            .territorycount = 1;
        stats.entry(3).or_insert_with(|| Stats::new(5, 3)).fives = 1;
        let territory_stats: Vec<TerritoryStats> = vec![
            TerritoryStats {
                team: 3,
                turn_id: 4,
                territory: 2,
                territory_power: 0.0,
                chance: 0.0,
                teampower: 0.0,
                fives: 1,
                ..TerritoryStats::default()
            },
            TerritoryStats {
                team: 6,
                turn_id: 4,
                territory: 2,
                territory_power: 0.0,
                chance: 1.0,
                teampower: 0.0,
                ..TerritoryStats::default()
            },
        ];

        // Because we are only testing functionality, not actually running ringmaster
        // we pass a known seed for reproducible results.
        assert_eq!(
            (new_owners, mvps, stats, territory_stats),
            process_territories(
                territories,
                playermoves,
                &mut ChaCha12Rng::seed_from_u64(45),
                true,
                Vec::new()
            )
        );
    }

    #[test]
    fn test_process_territories_one_same_alt() {
        let territories = vec![TerritoryOwners {
            id: 1,
            territory_id: 2,
            owner_id: 3,
            turn_id: 4,
            previous_owner_id: 5,
            random_number: 0.0,
            mvp: None,
            is_respawn: false,
        }];

        let playermoves: Vec<PlayerMoves> = vec![PlayerMoves {
            id: 45,
            user_id: 6,
            turn_id: 4,
            territory: 2,
            mvp: false,
            power: 0.0,
            multiplier: 1.0,
            weight: 12.0,
            stars: 5,
            team: 3,
            alt_score: 80,
            merc: false,
        }];
        let new_owners = vec![TerritoryOwnersInsert::new(
            &territories[0],
            3,
            Some(0.0),
            None,
            false,
        )];
        let mvps: Vec<PlayerMoves> = Vec::new();
        let mut stats: BTreeMap<i32, Stats> = BTreeMap::new();
        stats
            .entry(3)
            .or_insert_with(|| Stats::new(5, 3))
            .territorycount = 1;
        let territory_stats: Vec<TerritoryStats> = vec![TerritoryStats {
            team: 3,
            turn_id: 4,
            territory: 2,
            ..TerritoryStats::default()
        }];

        // Because we are only testing functionality, not actually running ringmaster
        // we pass a known seed for reproducible results.
        assert_eq!(
            (new_owners, mvps, stats, territory_stats),
            process_territories(
                territories,
                playermoves,
                &mut ChaCha12Rng::seed_from_u64(45),
                true,
                Vec::new()
            )
        );
    }

    #[test]
    fn test_process_territories_two() {
        let territories = vec![TerritoryOwners {
            id: 1,
            territory_id: 2,
            owner_id: 3,
            turn_id: 4,
            previous_owner_id: 5,
            random_number: 0.0,
            mvp: Some(12),
            is_respawn: false,
        }];

        let playermoves: Vec<PlayerMoves> = vec![
            PlayerMoves {
                id: 45,
                user_id: 6,
                turn_id: 4,
                territory: 2,
                mvp: false,
                power: 12.0,
                multiplier: 1.0,
                weight: 12.0,
                stars: 5,
                team: 3,
                alt_score: 0,
                merc: false,
            },
            PlayerMoves {
                id: 46,
                user_id: 7,
                turn_id: 4,
                territory: 2,
                mvp: false,
                power: 5.0,
                multiplier: 1.0,
                weight: 5.0,
                stars: 4,
                team: 2,
                alt_score: 0,
                merc: false,
            },
        ];
        let new_owners = vec![TerritoryOwnersInsert::new(
            &territories[0],
            3,
            Some(5.254_147_072_201_107),
            Some(6),
            false,
        )];
        let mvps: Vec<PlayerMoves> = vec![playermoves[0].clone()];
        let mut stats: BTreeMap<i32, Stats> = BTreeMap::new();
        stats
            .entry(2)
            .or_insert_with(|| Stats::new(5, 2))
            .territorycount = 0;
        stats
            .entry(2)
            .or_insert_with(|| Stats::new(5, 2))
            .playercount = 1;
        stats.entry(2).or_insert_with(|| Stats::new(5, 2)).starpower = 5.0;
        stats
            .entry(2)
            .or_insert_with(|| Stats::new(5, 2))
            .efficiency = 0.0;
        stats
            .entry(2)
            .or_insert_with(|| Stats::new(5, 2))
            .effectivepower = 5.0;
        stats.entry(2).or_insert_with(|| Stats::new(5, 3)).fours = 1;
        stats
            .entry(3)
            .or_insert_with(|| Stats::new(5, 3))
            .territorycount = 1;
        stats
            .entry(3)
            .or_insert_with(|| Stats::new(5, 3))
            .playercount = 1;
        stats.entry(3).or_insert_with(|| Stats::new(5, 3)).starpower = 12.0;
        stats
            .entry(3)
            .or_insert_with(|| Stats::new(5, 3))
            .efficiency = 0.0;
        stats
            .entry(3)
            .or_insert_with(|| Stats::new(5, 3))
            .effectivepower = 12.0;
        stats.entry(3).or_insert_with(|| Stats::new(5, 3)).fives = 1;
        let territory_stats: Vec<TerritoryStats> = vec![
            TerritoryStats {
                team: 2,
                turn_id: 4,
                territory: 2,
                territory_power: 17.0,
                chance: 0.294_117_647_058_823_54,
                teampower: 5.0,
                fours: 1,
                ..TerritoryStats::default()
            },
            TerritoryStats {
                team: 3,
                turn_id: 4,
                territory: 2,
                territory_power: 17.0,
                chance: 0.705_882_352_941_176_5,
                teampower: 12.0,
                fives: 1,
                ..TerritoryStats::default()
            },
        ];

        // Because we are only testing functionality, not actually running ringmaster
        // we pass a known seed for reproducible results.
        assert_eq!(
            (new_owners, mvps, stats, territory_stats),
            process_territories(
                territories,
                playermoves,
                &mut ChaCha12Rng::seed_from_u64(45),
                true,
                Vec::new()
            )
        );
    }

    #[test]
    fn test_process_territories_two_powerless() {
        let territories = vec![TerritoryOwners {
            id: 1,
            territory_id: 2,
            owner_id: 6,
            turn_id: 4,
            previous_owner_id: 5,
            random_number: 0.0,
            mvp: Some(12),
            is_respawn: false,
        }];

        let playermoves: Vec<PlayerMoves> = vec![
            PlayerMoves {
                id: 45,
                user_id: 6,
                turn_id: 4,
                territory: 2,
                mvp: false,
                power: 0.0,
                multiplier: 0.0,
                weight: 12.0,
                stars: 5,
                team: 3,
                alt_score: 0,
                merc: false,
            },
            PlayerMoves {
                id: 46,
                user_id: 7,
                turn_id: 4,
                territory: 2,
                mvp: false,
                power: 0.0,
                multiplier: 0.0,
                weight: 5.0,
                stars: 4,
                team: 2,
                alt_score: 170,
                merc: false,
            },
            PlayerMoves {
                id: 46,
                user_id: 17,
                turn_id: 4,
                territory: 2,
                mvp: false,
                power: 0.0,
                multiplier: 0.0,
                weight: 5.0,
                stars: 4,
                team: 3,
                alt_score: 175,
                merc: false,
            },
            PlayerMoves {
                id: 46,
                user_id: 18,
                turn_id: 4,
                territory: 2,
                mvp: false,
                power: 0.0,
                multiplier: 0.0,
                weight: 5.0,
                stars: 2,
                team: 3,
                alt_score: 5,
                merc: false,
            },
            PlayerMoves {
                id: 46,
                user_id: 198,
                turn_id: 4,
                territory: 2,
                mvp: false,
                power: 0.0,
                multiplier: 0.0,
                weight: 5.0,
                stars: 1,
                team: 3,
                alt_score: 5,
                merc: false,
            },
            PlayerMoves {
                id: 46,
                user_id: 198,
                turn_id: 4,
                territory: 2,
                mvp: false,
                power: 0.0,
                multiplier: 0.0,
                weight: 5.0,
                stars: 3,
                team: 3,
                alt_score: 5,
                merc: false,
            },
        ];
        let new_owners = vec![TerritoryOwnersInsert::new(
            &territories[0],
            6,
            Some(0.0),
            None,
            false,
        )];
        let mvps: Vec<PlayerMoves> = vec![];
        let mut stats: BTreeMap<i32, Stats> = BTreeMap::new();
        stats
            .entry(6)
            .or_insert_with(|| Stats::new(5, 6))
            .territorycount = 1;
        stats
            .entry(3)
            .or_insert_with(|| Stats::new(5, 3))
            .playercount = 4;
        stats.entry(3).or_insert_with(|| Stats::new(5, 3)).starpower = 0.0;
        stats
            .entry(3)
            .or_insert_with(|| Stats::new(5, 3))
            .efficiency = 0.0;
        stats
            .entry(3)
            .or_insert_with(|| Stats::new(5, 3))
            .effectivepower = 0.0;
        // Not an alt so should be listed, just with 0 power.
        stats.entry(3).or_insert_with(|| Stats::new(5, 3)).fives = 1;
        stats.entry(3).or_insert_with(|| Stats::new(5, 3)).twos = 1;
        stats.entry(3).or_insert_with(|| Stats::new(5, 3)).ones = 1;
        stats.entry(3).or_insert_with(|| Stats::new(5, 3)).threes = 1;
        // Team 2 is exlcuded as it did not have any non-alts.
        // Team 3 is included because it tried.
        let territory_stats: Vec<TerritoryStats> = vec![
            TerritoryStats {
                team: 3,
                turn_id: 4,
                territory: 2,
                territory_power: 0.0,
                chance: 0.0,
                teampower: 0.0,
                fives: 1,
                twos: 1,
                ones: 1,
                threes: 1,
                ..TerritoryStats::default()
            },
            TerritoryStats {
                team: 6,
                turn_id: 4,
                territory: 2,
                territory_power: 0.0,
                chance: 1.0,
                teampower: 0.0,
                ..TerritoryStats::default()
            },
        ];

        // Because we are only testing functionality, not actually running ringmaster
        // we pass a known seed for reproducible results.
        assert_eq!(
            (new_owners, mvps, stats, territory_stats),
            process_territories(
                territories,
                playermoves,
                &mut ChaCha12Rng::seed_from_u64(45),
                true,
                Vec::new()
            )
        );
    }
}
