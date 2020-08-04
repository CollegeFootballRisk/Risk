use crate::schema::{new_turns, past_turns, stats, territory_ownership, territory_stats, turninfo};
use chrono::NaiveDateTime;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::result::Error;
use diesel::{insert_into, update};
use std::collections::HashMap;
#[derive(Deserialize, Insertable, Queryable, Debug, PartialEq, Clone)]
#[table_name = "new_turns"]
pub struct PlayerMoves {
    pub id: i32,
    pub user_id: i32,
    pub season: Option<i32>,
    pub day: Option<i32>,
    pub territory: i32,
    pub mvp: bool,
    pub power: f64,
    pub multiplier: Option<f64>,
    pub weight: Option<f64>,
    pub stars: i32,
    pub team: i32,
    pub alt_score: i32,
    pub merc: bool,
}

#[derive(Deserialize, Insertable, Queryable, Debug, PartialEq, Clone)]
#[table_name = "stats"]
pub struct Stats {
    pub sequence: i32,
    pub season: i32,
    pub day: i32,
    pub team: i32,
    pub rank: i32,
    pub territorycount: i32,
    pub playercount: i32,
    pub merccount: i32,
    pub starpower: i32,
    pub efficiency: i32,
    pub effectivepower: i32,
    pub ones: i32,
    pub twos: i32,
    pub threes: i32,
    pub fours: i32,
    pub fives: i32,
}

#[derive(Deserialize, Insertable, Queryable)]
#[table_name = "territory_ownership"]
pub struct TerritoryOwners {
    pub id: i32,
    pub territory_id: i32,
    pub territory_name: Option<String>,
    pub owner_id: i32,
    pub day: i32,
    pub season: i32,
    pub previous_owner_id: i32,
    pub random_number: f64,
    pub mvp: Option<i32>,
}

#[derive(Deserialize, Insertable, Queryable, Debug)]
#[table_name = "territory_ownership"]
pub struct TerritoryOwnersInsert {
    pub territory_id: i32,
    pub territory_name: Option<String>,
    pub owner_id: i32,
    pub day: i32,
    pub season: i32,
    pub previous_owner_id: i32,
    pub random_number: f64,
    pub mvp: Option<i32>,
}

#[derive(Deserialize, Insertable, Queryable, Debug, PartialEq, Clone)]
#[table_name = "territory_stats"]
pub struct TerritoryStats {
    pub team: i32,
    pub season: i32,
    pub day: i32,
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

#[derive(Deserialize, Insertable, Queryable, Debug, PartialEq, Clone)]
#[table_name = "turninfo"]
pub struct TurnInfo {
    pub id: i32,
    pub season: Option<i32>,
    pub day: Option<i32>,
    pub complete: Option<bool>,
    pub active: Option<bool>,
    pub finale: Option<bool>,
    pub chaosweight: Option<i32>,
    pub rollendtime: Option<NaiveDateTime>,
    pub rollstarttime: Option<NaiveDateTime>,
}

impl PlayerMoves {
    pub fn load(season: &i32, day: &i32, conn: &PgConnection) -> Result<Vec<PlayerMoves>, Error> {
        new_turns::table
            .filter(new_turns::season.eq(season))
            .filter(new_turns::day.eq(day))
            .order_by(new_turns::territory.desc())
            .load::<PlayerMoves>(conn)
    }

    pub fn mvps(mvps: Vec<PlayerMoves>, conn: &PgConnection) -> QueryResult<usize> {
        //first we flatten
        let mvp_array = mvps.iter().map(|x| x.id).collect::<Vec<i32>>();
        update(new_turns::table.filter(new_turns::id.eq_any(mvp_array)))
            .set(new_turns::mvp.eq(true))
            .execute(conn)
    }

    pub fn mergemoves(min: i32, max: i32, conn: &PgConnection) -> QueryResult<usize> {
        new_turns::table
            .select((
                new_turns::user_id,
                new_turns::season,
                new_turns::day,
                new_turns::territory,
                new_turns::mvp,
                new_turns::power,
                new_turns::multiplier,
                new_turns::weight,
                new_turns::stars,
                new_turns::team,
                new_turns::alt_score,
                new_turns::merc,
            ))
            .filter(new_turns::id.le(max))
            .filter(new_turns::id.ge(min))
            .insert_into(past_turns::table)
            .into_columns((
                past_turns::user_id,
                past_turns::season,
                past_turns::day,
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
    }
}

impl Stats {
    pub fn insert(
        stats: HashMap<i32, Stats>,
        sequence: i32,
        conn: &PgConnection,
    ) -> QueryResult<usize> {
        // calculate whichever has the highest number of territories and such
        let mut insertable_stats = stats.values().collect::<Vec<_>>();
        insertable_stats.sort_by_key(|a| a.territorycount);
        insertable_stats.reverse();
        let mut rankings: i32 = 1;
        let mut territories: i32 = 0;
        let mut amended_stats: Vec<Stats> = Vec::new();
        for i in insertable_stats.iter() {
            if i.territorycount < territories {
                rankings += 1;
            }
            territories = i.territorycount;
            amended_stats.push(Stats {
                sequence,
                season: i.season,
                day: i.day,
                team: i.team,
                rank: rankings,
                territorycount: i.territorycount,
                playercount: i.playercount,
                merccount: i.merccount,
                starpower: i.starpower,
                efficiency: i.starpower / i.territorycount,
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

    pub fn new(seq: i32, season: i32, day: i32, team: i32) -> Stats {
        Stats {
            sequence: seq,
            season,
            day,
            team,
            rank: 0,
            territorycount: 0,
            playercount: 0,
            merccount: 0,
            starpower: 0,
            efficiency: 0,
            effectivepower: 0,
            ones: 0,
            twos: 0,
            threes: 0,
            fours: 0,
            fives: 0,
        }
    }
}

impl TerritoryStats {
    pub fn insert(stats: Vec<TerritoryStats>, conn: &PgConnection) -> QueryResult<usize> {
        diesel::insert_into(territory_stats::table)
            .values(stats)
            .execute(conn)
    }
}

impl TerritoryOwners {
    pub fn load(
        season: &i32,
        day: &i32,
        conn: &PgConnection,
    ) -> Result<Vec<TerritoryOwners>, Error> {
        territory_ownership::table
            .filter(territory_ownership::season.eq(season))
            .filter(territory_ownership::day.eq(day))
            .load::<TerritoryOwners>(conn)
    }
}

impl TerritoryOwnersInsert {
    pub fn insert(owners: Vec<TerritoryOwnersInsert>, conn: &PgConnection) -> QueryResult<usize> {
        use crate::schema::territory_ownership::dsl::*;
        insert_into(territory_ownership)
            .values(&owners)
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
            ))
            .execute(conn)
    }

    pub fn insert_new(
        season: i32,
        day: i32,
        active: bool,
        finale: bool,
        conn: &PgConnection,
    ) -> QueryResult<usize> {
        //use schema::turninfo::dsl::*;
        diesel::insert_into(turninfo::table)
            .values((
                turninfo::season.eq(&Some(season)),
                turninfo::day.eq(&Some(day)),
                turninfo::complete.eq(&Some(false)),
                turninfo::active.eq(&Some(active)),
                turninfo::finale.eq(&Some(finale)),
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
            ))
            .filter(turninfo::active.eq(true))
            .order((turninfo::season.desc(), turninfo::day.desc()))
            .first::<TurnInfo>(conn)
    }
}
