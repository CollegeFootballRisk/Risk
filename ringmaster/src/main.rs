//TODO: Stats
#![feature(drain_filter)]
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate serde_derive;
extern crate dotenv;
extern crate rand;

use diesel::{insert_into, sql_query, update};

use chrono::{NaiveDateTime, Utc};
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::result::Error;
use rand::prelude::*;
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::Read;
use std::io::Write;

mod schema {
    table! {
        new_turns (id) {
            id -> Int4,
            user_id -> Int4,
            season -> Nullable<Int4>,
            day -> Nullable<Int4>,
            territory -> Int4,
            mvp -> Bool,
            power -> Double,
            multiplier -> Nullable<Double>,
            weight -> Nullable<Double>,
            stars -> Int4,
            team -> Int4,
            alt_score -> Int4,
            merc -> Bool,
        }
    }

    table! {
        past_turns (id) {
            id -> Int4,
            user_id -> Int4,
            season -> Nullable<Int4>,
            day -> Nullable<Int4>,
            territory -> Int4,
            mvp -> Bool,
            power -> Double,
            multiplier -> Nullable<Double>,
            weight -> Nullable<Double>,
            stars -> Int4,
            team -> Int4,
            alt_score -> Int4,
            merc -> Bool,
        }
    }

    table! {
        turninfo (id) {
            id -> Int4,
            season -> Nullable<Int4>,
            day -> Nullable<Int4>,
            complete -> Nullable<Bool>,
            active -> Nullable<Bool>,
            finale -> Nullable<Bool>,
            chaosrerolls -> Nullable<Int4>,
            chaosweight -> Nullable<Int4>,
            rollendtime -> Nullable<Timestamp>,
            rollstarttime -> Nullable<Timestamp>,
        }
    }

    table! {
        stats (sequence) {
            sequence -> Int4,
            season -> Int4,
            day -> Int4,
            team -> Int4,
            rank -> Int4,
            territorycount -> Int4,
            playercount -> Int4,
            merccount -> Int4,
            starpower -> Int4,
            efficiency -> Int4,
            effectivepower -> Int4,
            ones -> Int4,
            twos -> Int4,
            threes -> Int4,
            fours -> Int4,
            fives -> Int4,
        }
    }

    table! {
        territory_stats (id) {
            team -> Int4,
            season -> Int4,
            day -> Int4,
            ones -> Int4,
            twos -> Int4,
            threes -> Int4,
            fours -> Int4,
            fives -> Int4,
            teampower -> Double,
            chance -> Double,
            id -> Int4,
            territory -> Int4,
            territory_power -> Double,
        }
    }

    table! {
        territory_ownership (id) {
            id -> Int4,
            territory_id -> Int4,
            territory_name -> Nullable<Text>,
            owner_id -> Int4,
            day -> Int4,
            season -> Int4,
            previous_owner_id -> Int4,
            random_number -> Double,
            mvp -> Nullable<Int4>,
        }
    }
}
use schema::{new_turns, past_turns, territory_ownership, turninfo, stats, territory_stats};

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

impl PlayerMoves {
    fn load(season: &i32, day: &i32, conn: &PgConnection) -> Result<Vec<PlayerMoves>, Error> {
        new_turns::table
            .filter(new_turns::season.eq(season))
            .filter(new_turns::day.eq(day))
            .order_by(new_turns::territory.desc())
            .load::<PlayerMoves>(conn)
    }

    fn mvps(mvps: Vec<PlayerMoves>, conn: &PgConnection) -> QueryResult<usize> {
        //first we flatten
        let mvp_array = mvps.iter().map(|x| x.id).collect::<Vec<i32>>();
        update(new_turns::table.filter(new_turns::id.eq_any(mvp_array)))
            .set(new_turns::mvp.eq(true))
            .execute(conn)
    }

    fn mergemoves(min: i32, max: i32, conn: &PgConnection) -> QueryResult<usize> {
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
    pub fives: i32
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
    pub territory_power: f64
}


impl TerritoryStats {
    fn insert(stats: Vec<TerritoryStats>, conn: &PgConnection) -> QueryResult<usize> {
        diesel::insert_into(territory_stats::table)
        .values(stats)
        .execute(conn)
    }
}

impl Stats {
    fn insert(stats: HashMap<i32, Stats>, sequence: i32, conn: &PgConnection) -> QueryResult<usize> {
        //dbg!(&stats);
        // calculate whichever has the highest number of territories and such
        let mut insertable_stats = stats.values().collect::<Vec<_>>();
        insertable_stats.sort_by_key(|a| a.territorycount);
        insertable_stats.reverse();
        let mut rankings:i32 = 1;
        let mut territories: i32 = 0;
        let mut amended_stats: Vec<Stats> = Vec::new();
        for i in insertable_stats.iter() {
            if i.territorycount > territories{
                let rank = rankings;
            }
            else {
                rankings += 1;
                let rank = rankings;
            }
            territories = i.territorycount;
            amended_stats.push(Stats {
                sequence: sequence,
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
                fives: i.fives
            } );
        }
        diesel::insert_into(stats::table)
        .values(amended_stats)
        .execute(conn)
    }

    fn new(seq: i32, season: i32, day: i32, team: i32) -> Stats {
        Stats {
            sequence: seq,
            season: season,
            day: day,
            team: team,
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
            fives: 0
        }
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

    pub fn new(
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

impl TerritoryOwnersInsert {
    fn insert(owners: Vec<TerritoryOwnersInsert>, conn: &PgConnection) -> QueryResult<usize> {
        use schema::territory_ownership::dsl::*;
        insert_into(territory_ownership)
            .values(&owners)
            .execute(conn)
    }
}

pub struct Team {
    pub number_of_players: i32,
    pub star_power: i32,
    pub victors: bool,
}

impl TerritoryOwners {
    fn load(season: &i32, day: &i32, conn: &PgConnection) -> Result<Vec<TerritoryOwners>, Error> {
        territory_ownership::table
            .filter(territory_ownership::season.eq(season))
            .filter(territory_ownership::day.eq(day))
            .load::<TerritoryOwners>(conn)
    }
}

pub fn establish_connection() -> PgConnection {
    dotenv::from_filename("../.env").ok();
    let database_url = dotenv::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url).expect(&format!("Error connecting to {}", database_url))
}

fn getteams(territory_players: Vec<PlayerMoves>) -> Vec<i32> {
    let mut teams = territory_players
        .iter()
        .map(|x| x.team)
        .collect::<Vec<i32>>();
    teams.sort_by(|a, b| a.cmp(&b));
    teams.dedup();
    teams
}

fn determinevictor(lottery: f64, map: HashMap<i32, (i32, f64, i32, i32, i32, i32, i32)>) -> i32 {
    let mut victorsum = 0f64;
    //println!("Map: {:?}", map);
    let mut victor = 0;
    for (key, val) in &map {
        //println!("Key {:?} Val {:?}",key,val);
        victorsum += val.1;
        if lottery - victorsum < 0f64 {
            victor = *key;
            break;
        }
    }
    victor
}

fn getmvp(mut territory_players: Vec<PlayerMoves>) -> PlayerMoves {
    let rng = match territory_players.len() {
        1 => 0,
        _ => rand::thread_rng().gen_range(1, territory_players.len()),
    };
    territory_players.remove(rng)
}

fn process_territories<'a>(
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
        //dbg!(&territory.territory_id);
        let mut territory_players = players
            .drain_filter(|player| player.territory == territory.territory_id)
            .collect::<Vec<_>>();
            //dbg!(&territory_players.len());
        let teams = getteams(territory_players.clone());
        match teams.len() {
            0 => {
                new_owners.push(TerritoryOwnersInsert {
                    territory_id: territory.territory_id,
                    territory_name: None,
                    owner_id: territory.owner_id.clone(),
                    day: territory.day + 1,
                    season: territory.season,
                    previous_owner_id: territory.owner_id,
                    random_number: 0f64,
                    mvp: Some(0),
                });
                // add team territory count to stats
                stats
                .entry(territory.owner_id)
                .or_insert(Stats::new(
                    territory.season*1000 + territory.season + 1,
                    territory.season,
                     territory.day,
                      territory.owner_id))
                .territorycount += 1;

                territory_stats.push(
                    TerritoryStats{
                        team: territory.owner_id.clone(),
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
                    }
                );
                continue;
            }
            1 => {
                let mvp = getmvp(territory_players.clone());
                new_owners.push(TerritoryOwnersInsert {
                    territory_id: territory.territory_id,
                    territory_name: None,
                    owner_id: teams[0],
                    day: territory.day + 1,
                    season: territory.season,
                    previous_owner_id: territory.owner_id,
                    random_number: 0f64,
                    mvp: Some(mvp.user_id),
                });
                stats
                .entry(territory.owner_id)
                .or_insert(Stats::new(
                    territory.season*1000 + territory.season + 1,
                    territory.season,
                    territory.day,
                    teams[0]))
                .territorycount += 1;
                stats
                .entry(territory.owner_id)
                .or_insert(Stats::new(
                    territory.season*1000 + territory.season + 1,
                    territory.season,
                     territory.day,
                     teams[0]))
                .starpower += territory_players.iter().map(|mover| mover.power.round() as i32).sum::<i32>();
                // add team stats
                handleteamstats(&mut stats,territory_players.clone());
                territory_stats.push(
                    TerritoryStats{
                        team: territory.owner_id.clone(),
                        season: territory.season,
                        day: territory.day, 
                        ones: territory_players.iter().filter(|player| player.stars == 1).count() as i32,
                        twos: territory_players.iter().filter(|player| player.stars == 2).count() as i32,
                        threes: territory_players.iter().filter(|player| player.stars == 3).count() as i32,
                        fours: territory_players.iter().filter(|player| player.stars == 4).count() as i32,
                        fives: territory_players.iter().filter(|player| player.stars == 5).count() as i32,
                        teampower: territory_players.iter().map(|mover| mover.power.round() as f64).sum::<f64>(),
                        chance: 1.00,
                        territory: territory.territory_id,
                        territory_power: territory_players.iter().map(|mover| mover.power.round() as f64).sum::<f64>(),
                    }
                );
                continue;
            }
            _ => {
                let mut map = HashMap::new();
                for team in teams {
                    map.insert(team, (0, 0f64, 0, 0, 0, 0, 0)); // stars, power, ones, twos, threes, fours, fives
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
                let lottery = rand::thread_rng().gen_range(0f64, totalpower);

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
                    owner_id: victor.clone(),
                    day: territory.day + 1,
                    season: territory.season,
                    previous_owner_id: territory.owner_id,
                    random_number: lottery,
                    mvp: Some(mvp.user_id),
                });

                stats
                .entry(victor)
                .or_insert(Stats::new(
                    territory.season*1000 + territory.season + 1,
                    territory. season,
                     territory.day,
                     victor))
                .territorycount += 1;

                let total_power = territory_players.iter().map(|mover| mover.power.round() as f64).sum::<f64>();
                handleteamstats(&mut stats, territory_players);
                for (key, val) in map.iter(){
                    territory_stats.push(
                        TerritoryStats{
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
                            territory_power: total_power
                        }
                    );
                }
                mvps.push(mvp);
            }
        }
    }
    (new_owners, mvps, stats, territory_stats)
}

fn handleteamstats(stats: &mut HashMap<i32, Stats>,territory_players: Vec<PlayerMoves>) {
    //dbg!("wallop");
    //dbg!(&territory_players.len());
    for i in territory_players {
        stats
        .entry(i.team)
        .or_insert(Stats::new(i.season.unwrap_or(0)*1000 + i.day.unwrap_or(0) + 1, i.season.unwrap_or(0), i.day.unwrap_or(0), i.team))
        .playercount += 1;

        stats
        .entry(i.team)
        .or_insert(Stats::new(i.season.unwrap_or(0)*1000 + i.day.unwrap_or(0) + 1, i.season.unwrap_or(0), i.day.unwrap_or(0), i.team))
        .starpower += i.stars;

        stats
        .entry(i.team)
        .or_insert(Stats::new(i.season.unwrap_or(0)*1000 + i.day.unwrap_or(0) + 1, i.season.unwrap_or(0), i.day.unwrap_or(0), i.team))
        .effectivepower += i.power.round() as i32;

        if i.merc == true {
        stats
        .entry(i.team)
        .or_insert(Stats::new(i.season.unwrap_or(0)*1000 + i.day.unwrap_or(0) + 1, i.season.unwrap_or(0), i.day.unwrap_or(0), i.team))
        .merccount += 1;
        }
        match i.stars {
           1 => {
            stats
            .entry(i.team)
            .or_insert(Stats::new(i.season.unwrap_or(0)*1000 + i.day.unwrap_or(0) + 1, i.season.unwrap_or(0), i.day.unwrap_or(0), i.team))
            .ones += 1;
           }
           2 => {
            stats
            .entry(i.team)
            .or_insert(Stats::new(i.season.unwrap_or(0)*1000 + i.day.unwrap_or(0) + 1, i.season.unwrap_or(0), i.day.unwrap_or(0), i.team))
            .twos += 1;
           }
           3 => {
            stats
            .entry(i.team)
            .or_insert(Stats::new(i.season.unwrap_or(0)*1000 + i.day.unwrap_or(0) + 1, i.season.unwrap_or(0), i.day.unwrap_or(0), i.team))
            .threes += 1;
           }
           4 => {
            stats
            .entry(i.team)
            .or_insert(Stats::new(i.season.unwrap_or(0)*1000 + i.day.unwrap_or(0) + 1, i.season.unwrap_or(0), i.day.unwrap_or(0), i.team))
            .fours += 1;
           }
           5 => {
            stats
            .entry(i.team)
            .or_insert(Stats::new(i.season.unwrap_or(0)*1000 + i.day.unwrap_or(0) + 1, i.season.unwrap_or(0), i.day.unwrap_or(0), i.team))
            .fives += 1;
           }
           _ => {
            stats
            .entry(i.team)
            .or_insert(Stats::new(i.season.unwrap_or(0)*1000 + i.day.unwrap_or(0) + 1, i.season.unwrap_or(0), i.day.unwrap_or(0), i.team))
            .ones += 1;
           }
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
        let players = PlayerMoves::load(
            &turninfoblock.season.unwrap(),
            &turninfoblock.day.unwrap(),
            &conn,
        );
        let territories = TerritoryOwners::load(
            &turninfoblock.season.unwrap(),
            &turninfoblock.day.unwrap(),
            &conn,
        );
        let result = match territories {
            Ok(territories) => {
                match players {
                    Ok(players) => {
                        if players.len() > 0 {
                            let move_ids =
                                players.clone().iter().map(|x| x.id).collect::<Vec<i32>>();
                            let min_value = move_ids.iter().min();
                            let max_value = move_ids.iter().max();
                            dbg!(min_value, max_value, players.len());
                            let (owners, mvps, stats, territory_stats) = process_territories(territories, players);
                            //dbg!(&owners,&mvps, &stats);
                            //dbg!(&stats);
                            match TerritoryStats::insert(territory_stats, &conn){
                                Ok(result) => {
                                    match Stats::insert(stats, &turninfoblock.id + 0, &conn) {
                                        Ok(result) => {
                                            match TerritoryOwnersInsert::insert(owners, &conn) {
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
                                                                    };
                                                                    let query = format!(
                                                                        "SELECT do_user_update({},{})",
                                                                        &turninfoblock.day.unwrap().to_string(),
                                                                        &turninfoblock.season.unwrap().to_string()
                                                                    );
                                                                    let userupdate: Result<
                                                                        Vec<Bar>,
                                                                        diesel::result::Error,
                                                                    > = sql_query(query.to_string()).load(&conn);
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
                                                                        Some(Utc::now().naive_utc());
                                                                    turninfoblock.complete = Some(true);
                                                                    turninfoblock.active = Some(false);
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
                                                                    match TurnInfo::new(
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
                                                                        .open("../.env");
                                                                    let f_out = OpenOptions::new()
                                                                        .create(true)
                                                                        .write(true)
                                                                        .open("../.env");
                                                                    let mut buffer = String::new();
                                                                    let mut f_in = f_in.unwrap();
                                                                    f_in.read_to_string(&mut buffer);
                                                                    dotenv::from_filename("../.env").ok();
                                                                    let day = dotenv::var("day").unwrap();
                                                                    let new_day = &day.parse::<i32>().unwrap() + 1;
                                                                    buffer = buffer.replace(
                                                                        &format!("day={}", day.to_string()),
                                                                        &format!("day={}", new_day.to_string())[..],
                                                                    );
                                                                    f_out
                                                                        .unwrap()
                                                                        .write_all(buffer.as_bytes())
                                                                        .expect("error");
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
                                        Err(e) => {dbg!(e);}
                                    }
                                }
                                Err(e) => {dbg!(e);}
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
