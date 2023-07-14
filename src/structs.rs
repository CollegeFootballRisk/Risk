/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::schema::{team_statistic, team, territory_ownership, territory_statistic, turn, move_};
use crate::Utc;
use chrono::NaiveDateTime;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::result::Error;
use diesel::sql_types::Bool;
use diesel::{insert_into, update};
use std::collections::BTreeMap;

#[derive(QueryableByName)]
pub struct Bar {
    #[diesel(sql_type = Bool)]
    pub do_user_update: bool,
}
#[derive(Deserialize, Insertable, Queryable, Debug, PartialEq, Clone)]
#[diesel(table_name = move_)]
pub struct PlayerMoves {
    pub id: i32,
    pub user_id: i32,
    pub turn_id: i32,
    pub territory_id: i32,
    pub is_mvp: bool,
    pub power: f64,
    pub multiplier: f64,
    pub weight: f64,
    pub stars: i32,
    pub team_id: i32,
    pub alt_score: i32,
    pub is_merc: bool,
}

#[derive(Deserialize, Insertable, Queryable, Debug, PartialEq, Clone)]
#[diesel(table_name = team_statistic)]
pub struct Stats {
    pub turn_id: i32,
    pub team: i32,
    pub rank: i32,
    pub territory_count: i32,
    pub player_count: i32,
    pub merc_count: i32,
    pub starpower: f64,
    pub efficiency: f64,
    pub effective_power: f64,
    pub ones: i32,
    pub twos: i32,
    pub threes: i32,
    pub fours: i32,
    pub fives: i32,
}

#[derive(Deserialize, Queryable)]
pub struct Team {
    pub id: i32,
    pub color: String,
}

#[derive(Deserialize, Insertable, Queryable)]
#[diesel(table_name = territory_ownership)]
pub struct TerritoryOwners {
    pub id: i32,
    pub territory_id: i32,
    pub owner_id: i32,
    pub turn_id: i32,
    pub previous_owner_id: i32,
    pub random_number: f64,
    pub mvp: Option<i32>,
}

#[derive(Deserialize, Insertable, Queryable, Debug, PartialEq)]
#[diesel(table_name = territory_ownership)]
pub struct TerritoryOwnersInsert {
    pub territory_id: i32,
    pub owner_id: i32,
    pub turn_id: i32,
    pub previous_owner_id: i32,
    pub random_number: f64,
    pub mvp: Option<i32>,
}

#[derive(Deserialize, Insertable, Queryable, Debug, PartialEq, Clone)]
#[diesel(table_name = territory_statistic)]
pub struct TerritoryStats {
    pub team: i32,
    pub turn_id: i32,
    pub ones: i32,
    pub twos: i32,
    pub threes: i32,
    pub fours: i32,
    pub fives: i32,
    pub teampower: f64,
    pub chance: f64,
    pub territory: i32,
    pub territory_power: f64,
}

#[derive(Deserialize, Insertable, Queryable, Debug, PartialEq, Eq, Clone)]
#[diesel(table_name = turn)]
pub struct TurnInfo {
    pub id: i32,
    pub season: i32,
    pub day: i32,
    pub complete: Option<bool>,
    pub active: Option<bool>,
    pub finale: Option<bool>,
    pub roll_end: Option<NaiveDateTime>,
    pub roll_start: Option<NaiveDateTime>,
    pub all_or_nothing: Option<bool>,
    pub map: Option<String>,
}

#[derive(Clone)]
pub struct Victor {
    pub stars: i32,
    pub power: f64,
    pub ones: i32,
    pub twos: i32,
    pub threes: i32,
    pub fours: i32,
    pub fives: i32,
}

impl Default for Victor {
    fn default() -> Self {
        Self {
            stars: 0,
            power: 0.0,
            ones: 0,
            twos: 0,
            threes: 0,
            fours: 0,
            fives: 0,
        }
    }
}

impl Victor {
    pub fn power(&mut self, power: f64) -> &mut Self {
        self.power += power;
        self
    }

    pub fn stars(&mut self, stars: i32) -> &mut Self {
        self.stars += stars;
        match stars {
            1 => self.ones += 1,
            2 => self.twos += 1,
            3 => self.threes += 1,
            4 => self.fours += 1,
            5 => self.fives += 1,
            _ => {
                println!("Possible error, OOB stars");
                self.ones += 1;
            }
        }
        self
    }
}

impl PlayerMoves {
    pub fn load(turn_id: &i32, conn: &mut PgConnection) -> Result<Vec<PlayerMoves>, Error> {
        move_::table
            .filter(move_::turn_id.eq(turn_id))
            .order_by(move_::territory_id.desc())
            .load::<PlayerMoves>(conn)
    }

    pub fn mvps(mvps: Vec<PlayerMoves>, conn: &mut PgConnection) -> QueryResult<usize> {
        //first we flatten
        let mvp_array = mvps.iter().map(|x| x.id).collect::<Vec<i32>>();
        update(move_::table.filter(move_::id.eq_any(mvp_array)))
            .set(move_::is_mvp.eq(true))
            .execute(conn)
    }
}

impl Stats {
    pub fn insert(
        statistic: BTreeMap<i32, Stats>,
        turn_id: i32,
        conn: &mut PgConnection,
    ) -> QueryResult<usize> {
        // calculate whichever has the highest number of territories and such
        let mut insertable_statistic = statistic.values().collect::<Vec<_>>();
        insertable_statistic.sort_by_key(|a| a.territory_count);
        insertable_statistic.reverse();
        let mut rankings: i32 = 1;
        let mut territories: i32 = 0;
        let mut next_ranking: i32 = 1;
        let mut amended_statistic: Vec<Stats> = Vec::new();
        for i in &insertable_statistic {
            // Do not count the 'NCAA' (placeholder for empty territories).
            if i.team == 0 {
                continue;
            }
            // if there are more territories, then team are not tied; increment +1
            if i.territory_count < territories {
                rankings = next_ranking;
            }
            // increment the rank counter
            next_ranking += 1;
            territories = i.territory_count;
            let teamefficiency: f64 = match territories {
                0 => 0.0,
                _ => i.starpower / f64::from(i.territory_count),
            };
            amended_statistic.push(Stats {
                turn_id,
                team: i.team,
                rank: rankings,
                territory_count: i.territory_count,
                player_count: i.player_count,
                merc_count: i.merc_count,
                starpower: i.starpower,
                efficiency: teamefficiency,
                effective_power: i.effective_power,
                ones: i.ones,
                twos: i.twos,
                threes: i.threes,
                fours: i.fours,
                fives: i.fives,
            });
        }
        diesel::insert_into(team_statistic::table)
            .values(amended_statistic)
            .execute(conn)
    }

    #[must_use]
    pub fn new(turn_id: i32, team: i32) -> Stats {
        Stats {
            turn_id,
            team,
            rank: 0,
            territory_count: 0,
            player_count: 0,
            merc_count: 0,
            starpower: 0.0,
            efficiency: 0.0,
            effective_power: 0.0,
            ones: 0,
            twos: 0,
            threes: 0,
            fours: 0,
            fives: 0,
        }
    }

    pub fn stars(&mut self, stars: i32) {
        match stars {
            1 => self.ones += 1,
            2 => self.twos += 1,
            3 => self.threes += 1,
            4 => self.fours += 1,
            5 => self.fives += 1,
            _ => {
                println!("Possible error, OOB stars");
                self.ones += 1
            }
        }
    }

    pub fn starpower(&mut self, starpower: f64) -> &mut Self {
        self.starpower += starpower;
        self
    }

    pub fn effective_power(&mut self, effective_power: f64) -> &mut Self {
        self.effective_power += effective_power;
        self
    }

    pub fn add_player_or_merc(&mut self, merc: bool) -> &mut Self {
        if merc {
            self.merc_count += 1;
        } else {
            self.player_count += 1;
        }
        self
    }
}

impl Team {
    pub fn load(conn: &mut PgConnection) -> Result<Vec<Team>, Error> {
        team::table
            .select((team::id, team::primary_color))
            .load::<Team>(conn)
    }
}

impl TerritoryStats {
    pub fn insert(statistic: Vec<TerritoryStats>, conn: &mut PgConnection) -> QueryResult<usize> {
        diesel::insert_into(territory_statistic::table)
            .values(statistic)
            .execute(conn)
    }
}

impl Default for TerritoryStats {
    fn default() -> Self {
        Self {
            team: 0,
            turn_id: 0,
            ones: 0,
            twos: 0,
            threes: 0,
            fours: 0,
            fives: 0,
            teampower: 0.0,
            chance: 1.00,
            territory: 0,
            territory_power: 0.00,
        }
    }
}

impl TerritoryOwners {
    pub fn load(turn_id: &i32, conn: &mut PgConnection) -> Result<Vec<TerritoryOwners>, Error> {
        territory_ownership::table
            .filter(territory_ownership::turn_id.eq(turn_id))
            .load::<TerritoryOwners>(conn)
    }
}

impl TerritoryOwnersInsert {
    #[must_use] pub fn new(
        territory: &TerritoryOwners,
        owner: i32,
        random_number: Option<f64>,
        mvp: Option<i32>,
    ) -> Self {
        TerritoryOwnersInsert {
            territory_id: territory.territory_id,
            owner_id: owner,
            turn_id: territory.turn_id + 1,
            previous_owner_id: territory.owner_id,
            random_number: random_number.unwrap_or(0_f64),
            mvp,
        }
    }

    pub fn insert(owners: &[TerritoryOwnersInsert], conn: &mut PgConnection) -> QueryResult<usize> {
        use crate::schema::territory_ownership::dsl::territory_ownership;
        insert_into(territory_ownership)
            .values(owners)
            .execute(conn)
    }
}

impl TurnInfo {
    pub fn update_or_insert(newturn: &Self, conn: &mut PgConnection) -> QueryResult<usize> {
        //use schema::turn::dsl::*;
        diesel::insert_into(turn::table)
            .values(newturn)
            .on_conflict(turn::id)
            .do_update()
            .set((
                turn::complete.eq(newturn.complete),
                turn::active.eq(newturn.active),
                turn::roll_start.eq(newturn.roll_start),
                turn::roll_end.eq(newturn.roll_end),
                turn::map.eq(&newturn.map),
                turn::all_or_nothing.eq(newturn.all_or_nothing),
            ))
            .execute(conn)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn insert_new(
        season: i32,
        day: i32,
        active: bool,
        finale: bool,
        map: Option<String>,
        all_or_nothing: bool,
        start_time: Option<NaiveDateTime>,
        conn: &mut PgConnection,
    ) -> QueryResult<usize> {
        //use schema::turn::dsl::*;
        diesel::insert_into(turn::table)
            .values((
                turn::season.eq(season),
                turn::day.eq(day),
                turn::complete.eq(&Some(false)),
                turn::active.eq(&Some(active)),
                turn::finale.eq(&Some(finale)),
                turn::map.eq(&map),
                turn::roll_start.eq(&start_time),
                turn::all_or_nothing.eq(&Some(all_or_nothing)),
            ))
            .on_conflict((turn::season, turn::day))
            .do_update()
            .set((
                turn::active.eq(&Some(active)),
                turn::complete.eq(&Some(false)),
                turn::finale.eq(&Some(finale)),
                turn::map.eq(&map),
                turn::roll_start.eq(&start_time),
                turn::all_or_nothing.eq(&Some(all_or_nothing)),
            ))
            .execute(conn)
    }

    pub fn get_latest(conn: &mut PgConnection) -> Result<TurnInfo, diesel::result::Error> {
        turn::table
            .select((
                turn::id,
                turn::season,
                turn::day,
                turn::complete,
                turn::active,
                turn::finale,
                turn::roll_end,
                turn::roll_start,
                turn::all_or_nothing,
                turn::map,
            ))
            .filter(turn::active.eq(true))
            .order((turn::season.desc(), turn::day.desc()))
            .first::<TurnInfo>(conn)
    }

    pub fn start_time_now(&mut self) -> &mut Self {
        self.roll_start = Some(Utc::now().naive_utc());
        self
    }

    pub fn lock(&mut self, conn: &mut PgConnection) -> Result<usize, Error> {
        update(turn::table.filter(turn::id.eq(self.id)))
            .set(turn::active.eq(false))
            .execute(conn)
    }
}
