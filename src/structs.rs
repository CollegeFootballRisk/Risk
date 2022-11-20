/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::schema::{stats, teams, territory_ownership, territory_stats, turninfo, turns};
use crate::Utc;
use chrono::NaiveDateTime;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::result::Error;
use diesel::sql_types::Bool;
use diesel::{insert_into, update};
use std::collections::HashMap;

#[derive(QueryableByName)]
pub struct Bar {
    #[sql_type = "Bool"]
    pub do_user_update: bool,
}
#[derive(Deserialize, Insertable, Queryable, Debug, PartialEq, Clone)]
#[table_name = "turns"]
pub struct PlayerMoves {
    pub id: i32,
    pub user_id: i32,
    pub turn_id: i32,
    pub territory: i32,
    pub mvp: bool,
    pub power: f64,
    pub multiplier: f64,
    pub weight: f64,
    pub stars: i32,
    pub team: i32,
    pub alt_score: i32,
    pub merc: bool,
}

#[derive(Deserialize, Insertable, Queryable, Debug, PartialEq, Clone)]
#[table_name = "stats"]
pub struct Stats {
    pub turn_id: i32,
    pub team: i32,
    pub rank: i32,
    pub territorycount: i32,
    pub playercount: i32,
    pub merccount: i32,
    pub starpower: f64,
    pub efficiency: f64,
    pub effectivepower: f64,
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
#[table_name = "territory_ownership"]
pub struct TerritoryOwners {
    pub id: i32,
    pub territory_id: i32,
    pub owner_id: i32,
    pub turn_id: i32,
    pub previous_owner_id: i32,
    pub random_number: f64,
    pub mvp: Option<i32>,
}

#[derive(Deserialize, Insertable, Queryable, Debug)]
#[table_name = "territory_ownership"]
pub struct TerritoryOwnersInsert {
    pub territory_id: i32,
    pub owner_id: i32,
    pub turn_id: i32,
    pub previous_owner_id: i32,
    pub random_number: f64,
    pub mvp: Option<i32>,
}

#[derive(Deserialize, Insertable, Queryable, Debug, PartialEq, Clone)]
#[table_name = "territory_stats"]
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
#[table_name = "turninfo"]
pub struct TurnInfo {
    pub id: i32,
    pub season: i32,
    pub day: i32,
    pub complete: Option<bool>,
    pub active: Option<bool>,
    pub finale: Option<bool>,
    pub chaosweight: Option<i32>,
    pub rollendtime: Option<NaiveDateTime>,
    pub rollstarttime: Option<NaiveDateTime>,
    pub allornothingenabled: Option<bool>,
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
    pub fn load(turn_id: &i32, conn: &PgConnection) -> Result<Vec<PlayerMoves>, Error> {
        turns::table
            .filter(turns::turn_id.eq(turn_id))
            .order_by(turns::territory.desc())
            .load::<PlayerMoves>(conn)
    }

    pub fn mvps(mvps: Vec<PlayerMoves>, conn: &PgConnection) -> QueryResult<usize> {
        //first we flatten
        let mvp_array = mvps.iter().map(|x| x.id).collect::<Vec<i32>>();
        update(turns::table.filter(turns::id.eq_any(mvp_array)))
            .set(turns::mvp.eq(true))
            .execute(conn)
    }

    /*pub fn mergemoves(min: i32, max: i32, conn: &PgConnection) -> QueryResult<usize> {
        turns::table
            .select((
                turns::user_id,
                turns::turn_id,
                turns::territory,
                turns::mvp,
                turns::power,
                turns::multiplier,
                turns::weight,
                turns::stars,
                turns::team,
                turns::alt_score,
                turns::merc,
            ))
            .filter(turns::id.le(max))
            .filter(turns::id.ge(min))
            .insert_into(past_turns::table)
            .into_columns((
                past_turns::user_id,
                past_turns::turn_id,
                past_turns::territory,
                past_turns::mvp,
                past_turns::power,
                past_turns::multiplier,
                past_turns::weight,
                past_turns::stars,
                past_turns::team,
                past_turns::alt_score,
                past_turns::merc,
            ))
            .execute(conn)
    }*/
}

impl Stats {
    pub fn insert(
        stats: HashMap<i32, Stats>,
        turn_id: i32,
        conn: &PgConnection,
    ) -> QueryResult<usize> {
        // calculate whichever has the highest number of territories and such
        let mut insertable_stats = stats.values().collect::<Vec<_>>();
        insertable_stats.sort_by_key(|a| a.territorycount);
        insertable_stats.reverse();
        let mut rankings: i32 = 1;
        let mut territories: i32 = 0;
        let mut next_ranking: i32 = 1;
        let mut amended_stats: Vec<Stats> = Vec::new();
        for i in &insertable_stats {
            // if there are more territories, then teams are not tied; increment +1
            if i.territorycount < territories {
                rankings = next_ranking;
            }
            // increment the rank counter
            next_ranking += 1;
            territories = i.territorycount;
            let teamefficiency: f64 = match territories {
                0 => 0.0,
                _ => i.starpower / f64::from(i.territorycount),
            };
            amended_stats.push(Stats {
                turn_id,
                team: i.team,
                rank: rankings,
                territorycount: i.territorycount,
                playercount: i.playercount,
                merccount: i.merccount,
                starpower: i.starpower,
                efficiency: teamefficiency,
                effectivepower: i.effectivepower,
                ones: i.ones,
                twos: i.twos,
                threes: i.threes,
                fours: i.fours,
                fives: i.fives,
            });
        }
        diesel::insert_into(stats::table)
            .values(amended_stats)
            .execute(conn)
    }

    #[must_use]
    pub fn new(turn_id: i32, team: i32) -> Stats {
        Stats {
            turn_id,
            team,
            rank: 0,
            territorycount: 0,
            playercount: 0,
            merccount: 0,
            starpower: 0.0,
            efficiency: 0.0,
            effectivepower: 0.0,
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

    pub fn effectivepower(&mut self, effectivepower: f64) -> &mut Self {
        self.effectivepower += effectivepower;
        self
    }

    pub fn add_player_or_merc(&mut self, merc: bool) -> &mut Self {
        if merc {
            self.merccount += 1;
        } else {
            self.playercount += 1;
        }
        self
    }
}

impl Team {
    pub fn load(conn: &PgConnection) -> Result<Vec<Team>, Error> {
        teams::table
            .select((teams::id, teams::color_1))
            .load::<Team>(conn)
    }
}

impl TerritoryStats {
    pub fn insert(stats: Vec<TerritoryStats>, conn: &PgConnection) -> QueryResult<usize> {
        diesel::insert_into(territory_stats::table)
            .values(stats)
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
    pub fn load(turn_id: &i32, conn: &PgConnection) -> Result<Vec<TerritoryOwners>, Error> {
        territory_ownership::table
            .filter(territory_ownership::turn_id.eq(turn_id))
            .load::<TerritoryOwners>(conn)
    }
}

impl TerritoryOwnersInsert {
    pub fn new(
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

    pub fn insert(owners: &[TerritoryOwnersInsert], conn: &PgConnection) -> QueryResult<usize> {
        use crate::schema::territory_ownership::dsl::territory_ownership;
        insert_into(territory_ownership)
            .values(owners)
            .execute(conn)
    }
}

impl TurnInfo {
    pub fn update_or_insert(newturninfo: &Self, conn: &PgConnection) -> QueryResult<usize> {
        //use schema::turninfo::dsl::*;
        diesel::insert_into(turninfo::table)
            .values(newturninfo)
            .on_conflict(turninfo::id)
            .do_update()
            .set((
                turninfo::complete.eq(newturninfo.complete),
                turninfo::active.eq(newturninfo.active),
                turninfo::rollstarttime.eq(newturninfo.rollstarttime),
                turninfo::rollendtime.eq(newturninfo.rollendtime),
                turninfo::map.eq(&newturninfo.map),
                turninfo::allornothingenabled.eq(newturninfo.allornothingenabled),
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
        allornothingenabled: bool,
        start_time: NaiveDateTime,
        conn: &PgConnection,
    ) -> QueryResult<usize> {
        //use schema::turninfo::dsl::*;
        diesel::insert_into(turninfo::table)
            .values((
                turninfo::season.eq(season),
                turninfo::day.eq(day),
                turninfo::complete.eq(&Some(false)),
                turninfo::active.eq(&Some(active)),
                turninfo::finale.eq(&Some(finale)),
                turninfo::map.eq(&map),
                turninfo::rollstarttime.eq(&Some(start_time)),
                turninfo::allornothingenabled.eq(&Some(allornothingenabled)),
            ))
            .on_conflict((turninfo::season, turninfo::day))
            .do_update()
            .set((
                turninfo::active.eq(&Some(active)),
                turninfo::complete.eq(&Some(false)),
                turninfo::finale.eq(&Some(finale)),
                turninfo::map.eq(&map),
                turninfo::rollstarttime.eq(&start_time),
                turninfo::allornothingenabled.eq(&Some(allornothingenabled)),
            ))
            .execute(conn)
    }

    pub fn get_latest(conn: &PgConnection) -> Result<TurnInfo, diesel::result::Error> {
        turninfo::table
            .select((
                turninfo::id,
                turninfo::season,
                turninfo::day,
                turninfo::complete,
                turninfo::active,
                turninfo::finale,
                turninfo::chaosweight,
                turninfo::rollendtime,
                turninfo::rollstarttime,
                turninfo::allornothingenabled,
                turninfo::map,
            ))
            .filter(turninfo::active.eq(true))
            .order((turninfo::season.desc(), turninfo::day.desc()))
            .first::<TurnInfo>(conn)
    }

    pub fn start_time_now(&mut self) -> &mut Self {
        self.rollstarttime = Some(Utc::now().naive_utc());
        self
    }
}
