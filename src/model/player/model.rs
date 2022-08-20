/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use crate::model::team::TeamWithColors;
use crate::model::turn::{LastTurn, PastTurn};
use crate::model::{Colors, Ratings, Stats, Team, Turn};
use crate::schema::{
    award_info, awards, moves, past_turns, team_player_moves, teams, territories, turninfo, users,
};
use diesel::prelude::*;
use diesel::result::Error;
use diesel_citext::types::CiString;
use schemars::JsonSchema;

#[derive(Queryable, Serialize, Deserialize, JsonSchema)]
pub(crate) struct Award {
    name: String,
    info: String,
}

#[derive(Serialize)]
pub(crate) struct Player {
    pub(crate) id: i32,
    pub(crate) name: CiString,
    pub(crate) team: Team,
    pub(crate) ratings: Ratings,
    pub(crate) stats: Stats,
    pub(crate) turns: Vec<Turn>,
}

#[derive(Queryable, Serialize, Deserialize, JsonSchema)]
pub(crate) struct TeamPlayer {
    pub(crate) team: Option<CiString>,
    pub(crate) player: Option<CiString>,
    pub(crate) turnsPlayed: Option<i32>,
    pub(crate) mvps: Option<i32>,
    pub(crate) lastTurn: LastTurn,
}

#[derive(Queryable, Serialize, Deserialize, JsonSchema)]
pub(crate) struct TeamMerc {
    pub(crate) team: CiString,
    pub(crate) player: CiString,
    pub(crate) turnsPlayed: Option<i32>,
    pub(crate) mvps: Option<i32>,
    pub(crate) stars: Option<i32>,
}

#[derive(Queryable, Identifiable, Associations, Serialize, Deserialize, JsonSchema)]
pub struct User {
    pub(crate) id: i32,
    pub(crate) uname: CiString,
    pub(crate) platform: CiString,
    pub(crate) turns: Option<i32>,
    pub(crate) game_turns: Option<i32>,
    pub(crate) mvps: Option<i32>,
    pub(crate) streak: Option<i32>,
    //pub(crate) awards: Option<i32>, //    pub team: Option<String>
}

#[derive(Queryable, Serialize, Deserialize, JsonSchema)]
pub(crate) struct PlayerWithTurns {
    pub(crate) name: CiString,
    pub(crate) team: Option<TeamWithColors>,
    pub(crate) platform: CiString,
    pub(crate) ratings: Ratings,
    pub(crate) stats: Stats,
    pub(crate) turns: Vec<PastTurn>,
}

#[derive(Queryable, Serialize, Deserialize, JsonSchema, Debug)]
pub(crate) struct PlayerInTurns {
    pub(crate) team: Option<CiString>,
    pub(crate) player: Option<CiString>,
    pub(crate) stars: Option<i32>,
    pub(crate) weight: i32,
    pub(crate) multiplier: f64,
    pub(crate) mvp: Option<bool>,
    pub(crate) power: f64,
}

#[derive(Queryable, Serialize, Deserialize, JsonSchema)]
pub(crate) struct PlayerWithTurnsAndAdditionalTeam {
    pub(crate) name: CiString,
    pub(crate) team: Option<TeamWithColors>,
    pub(crate) active_team: Option<TeamWithColors>,
    pub(crate) platform: CiString,
    pub(crate) ratings: Ratings,
    pub(crate) stats: Stats,
    pub(crate) turns: Vec<PastTurn>,
    pub(crate) awards: Vec<Award>,
}

#[derive(Queryable, Serialize, Deserialize, JsonSchema)]
pub(crate) struct PlayerSummary {
    pub(crate) name: CiString,
    pub(crate) platform: CiString,
    pub(crate) team: Option<CiString>,
}

impl PlayerSummary {
    pub(crate) fn load(conn: &PgConnection) -> Result<Vec<PlayerSummary>, diesel::result::Error> {
        users::table
            .left_join(teams::table.on(teams::id.eq(users::playing_for)))
            .select((users::uname, users::platform, teams::tname.nullable()))
            .load::<PlayerSummary>(conn)
    }
}

impl PlayerWithTurnsAndAdditionalTeam {
    pub(crate) fn load(
        name: Vec<String>,
        team_assigned: bool,
        conn: &PgConnection,
    ) -> Option<PlayerWithTurnsAndAdditionalTeam> {
        let me = PlayerWithTurns::load(name.clone(), true, conn);
        match me.len() {
            0 => None,
            1 => {
                use diesel::dsl::not;
                let status_code: i32 = match team_assigned {
                    true => 0,
                    false => -1,
                };
                let ciName: Vec<CiString> =
                    name.iter().map(|x| CiString::from(x.clone())).collect();
                let awards: Vec<Award> = awards::table
                    .left_join(award_info::table)
                    .left_join(users::table)
                    .filter(users::uname.eq(&me[0].name))
                    .select((award_info::name, award_info::info))
                    .load(conn)
                    .unwrap_or_default();
                let results = users::table
                    .filter(users::uname.eq_any(ciName))
                    .filter(not(users::current_team.eq(status_code)))
                    .left_join(teams::table.on(teams::id.eq(users::playing_for)))
                    .select((
                        teams::tname.nullable(),
                        teams::color_1.nullable(),
                        teams::color_2.nullable(),
                    ))
                    .first::<Team>(conn);
                match results {
                    Ok(results) => Some(PlayerWithTurnsAndAdditionalTeam {
                        name: me[0].name.clone(),
                        team: me[0].team.clone(),
                        active_team: Some(TeamWithColors {
                            name: results.name,
                            colors: Colors {
                                primary: results.color_1.unwrap_or_else(|| String::from("#000")),
                                secondary: results.color_2.unwrap_or_else(|| String::from("#000")),
                            },
                        }),
                        platform: me[0].platform.clone(),
                        ratings: me[0].ratings.clone(),
                        stats: me[0].stats.clone(),
                        turns: me[0].turns.clone(),
                        awards,
                    }),
                    Err(_e) => Some(PlayerWithTurnsAndAdditionalTeam {
                        name: me[0].name.clone(),
                        team: None,
                        active_team: None,
                        platform: me[0].platform.clone(),
                        ratings: me[0].ratings.clone(),
                        stats: me[0].stats.clone(),
                        turns: me[0].turns.clone(),
                        awards,
                    }),
                }
            }
            _ => None,
        }
    }
}

impl PlayerWithTurns {
    pub(crate) fn load(
        name: Vec<String>,
        team_assigned: bool,
        conn: &PgConnection,
    ) -> Vec<PlayerWithTurns> {
        use diesel::dsl::not;
        let status_code: i32 = match team_assigned {
            true => 0,
            false => -1,
        };
        let ciName: Vec<CiString> = name.iter().map(|x| CiString::from(x.clone())).collect();
        let results = users::table
            .filter(users::uname.eq_any(ciName))
            .filter(not(users::current_team.eq(status_code)))
            .left_join(teams::table.on(teams::id.eq(users::current_team)))
            .select((
                (
                    users::id,
                    users::uname,
                    users::platform,
                    users::turns,
                    users::game_turns,
                    users::mvps,
                    users::streak,
                    //users::awards,
                ),
                (
                    teams::tname.nullable(),
                    teams::color_1.nullable(),
                    teams::color_2.nullable(),
                ),
            ))
            .load::<(User, Team)>(conn)
            .expect("Error loading users");
        let mut out = Vec::new();
        for user in results {
            let stats = Stats {
                totalTurns: user.0.turns.unwrap_or(0),
                gameTurns: user.0.game_turns.unwrap_or(0),
                mvps: user.0.mvps.unwrap_or(0),
                streak: user.0.streak.unwrap_or(0),
                //awards: user.0.awards.unwrap_or(0),
            };
            let users_turns = past_turns::table
                .filter(past_turns::user_id.eq(&user.0.id))
                .inner_join(teams::table.on(teams::id.eq(past_turns::team)))
                .inner_join(territories::table.on(territories::id.eq(past_turns::territory)))
                .inner_join(turninfo::table.on(turninfo::id.eq(past_turns::turn_id)))
                .select((
                    turninfo::season,
                    turninfo::day,
                    past_turns::stars,
                    past_turns::mvp,
                    territories::name,
                    teams::tname,
                ))
                .order(past_turns::turn_id.desc())
                .load::<PastTurn>(conn)
                .expect("Error loading user turns");
            let uwp = PlayerWithTurns {
                name: user.0.uname,
                team: Some(TeamWithColors {
                    name: user.1.name,
                    colors: Colors {
                        primary: user.1.color_1.unwrap_or_else(|| String::from("#000")),
                        secondary: user.1.color_2.unwrap_or_else(|| String::from("#000")),
                    },
                }),
                platform: user.0.platform,
                ratings: Ratings::load(&stats),
                stats,
                turns: users_turns,
            };
            out.push(uwp);
        }
        out
    }
}

impl TeamPlayer {
    pub(crate) fn load(
        tname: Vec<String>,
        conn: &PgConnection,
    ) -> Result<Vec<TeamPlayer>, diesel::result::Error> {
        let ciTname: Vec<CiString> = tname.iter().map(|x| CiString::from(x.clone())).collect();
        moves::table
            .filter(moves::tname.eq_any(ciTname))
            .select((
                moves::tname,
                moves::uname,
                moves::turns,
                moves::mvps,
                (moves::season, moves::day, moves::stars),
            ))
            .load::<TeamPlayer>(conn)
    }

    pub(crate) fn loadall(conn: &PgConnection) -> Result<Vec<TeamPlayer>, diesel::result::Error> {
        moves::table
            .select((
                moves::tname,
                moves::uname,
                moves::turns,
                moves::mvps,
                (moves::season, moves::day, moves::stars),
            ))
            .load::<TeamPlayer>(conn)
    }
}

impl TeamMerc {
    pub(crate) fn load_mercs(
        tname: Vec<String>,
        conn: &PgConnection,
    ) -> Result<Vec<TeamMerc>, diesel::result::Error> {
        let ciTname: Vec<CiString> = tname.iter().map(|x| CiString::from(x.clone())).collect();
        allow_tables_to_appear_in_same_query!(users, moves);
        use diesel::dsl::not;
        users::table
            .inner_join(teams::table.on(teams::id.eq(users::playing_for)))
            .filter(teams::tname.eq_any(ciTname))
            .filter(not(users::playing_for.eq(users::current_team)))
            .select((
                teams::tname,
                users::uname,
                users::turns,
                users::mvps,
                users::overall,
            ))
            .load::<TeamMerc>(conn)
    }
}

impl PlayerInTurns {
    pub(crate) fn load(
        season: &i32,
        day: &i32,
        territory: &str,
        conn: &PgConnection,
    ) -> Result<Vec<PlayerInTurns>, Error> {
        let ciTerritory = CiString::from(territory.to_owned());
        team_player_moves::table
            .select((
                team_player_moves::team,
                team_player_moves::player,
                team_player_moves::stars,
                team_player_moves::weight,
                team_player_moves::multiplier,
                team_player_moves::mvp,
                team_player_moves::power,
            ))
            .filter(team_player_moves::day.eq(day))
            .filter(team_player_moves::season.eq(season))
            .filter(team_player_moves::territory.eq(ciTerritory))
            .load::<PlayerInTurns>(conn)
    }
}

impl User {
    pub fn load(name: String, platform: String, conn: &PgConnection) -> Result<User, Error> {
        users::table
            .filter(users::uname.eq(CiString::from(name)))
            .filter(users::platform.eq(CiString::from(platform)))
            .select((
                users::id,
                users::uname,
                users::platform,
                users::turns,
                users::game_turns,
                users::mvps,
                users::streak,
                //users::awards,
            ))
            .first::<User>(conn)
    }
}
