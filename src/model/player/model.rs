/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use crate::model::team::TeamWithColors;
use crate::model::turn::{LastTurn, PastTurn};
use crate::model::{Colors, Ratings, Stats, Team, Turn, UserId};
use crate::schema::{award_info, award, move, turn, team, territory, turn, user};
use diesel::prelude::*;
use diesel::result::Error;

use schemars::JsonSchema;

#[derive(Queryable, Serialize, Deserialize, JsonSchema, Debug)]
/// An official distinction applied by the game moderators to distinguish
/// user for a given reson/purpose. Awards do not affect actual gameplay.
pub struct Award {
    /// Name of the award
    name: String,
    /// Brief description of the award
    info: String,
}

#[derive(Serialize)]
pub(crate) struct Player {
    pub(crate) id: i32,
    pub(crate) name: String,
    pub(crate) team: Team,
    pub(crate) ratings: Ratings,
    pub(crate) stats: Stats,
    pub(crate) turns: Vec<Turn>,
}

#[derive(Queryable, Serialize, Deserialize, JsonSchema)]
/// A brief summary of a Player on a Team to display on the Team's page
pub(crate) struct TeamPlayer {
    /// The name of the Player's main team
    pub(crate) team: Option<String>,
    /// The name of the Player
    pub(crate) player: Option<String>,
    /// The number of turns played by the Player in all seasons
    pub(crate) turnsPlayed: Option<i32>,
    /// The number of MVPs won by the Player in all seasons
    pub(crate) mvps: Option<i32>,
    /// The season and day of the most-recent turn in which the Player participated.
    pub(crate) lastTurn: LastTurn,
}

#[derive(Queryable, Serialize, Deserialize, JsonSchema)]
/// A brief summary of a Mercenary Player on a Team to display on the Team's page
pub(crate) struct TeamMerc {
    /// The name of the Player's main team
    pub(crate) team: String,
    /// The name of the Player
    pub(crate) player: String,
    /// The number of turns played by the Player in all seasons
    pub(crate) turnsPlayed: Option<i32>,
    /// The number of MVPs won by the Player in all seasons
    pub(crate) mvps: Option<i32>,
    /// The season and day of the most-recent turn in which the Player participated.
    pub(crate) stars: Option<i32>,
}

#[derive(Queryable, Identifiable, Serialize, Deserialize, JsonSchema)]
/// An internal-only representation of a Player
pub struct User {
    /// The internal identifier for the user
    pub(crate) id: i32,
    /// The internal username for the user
    pub(crate) uname: String,
    /// [DEPRECATED] The `platform` from which the user connects
    /// This will be removed in a later version of RR.
    // TODO: Deprecate
    pub(crate) platform: String,
    pub(crate) turns: Option<i32>,
    pub(crate) game_turns: Option<i32>,
    pub(crate) mvps: Option<i32>,
    pub(crate) streak: Option<i32>,
    pub(crate) is_alt: bool, //pub(crate) award: Option<i32>, //    pub team: Option<String>
}

#[derive(Queryable, Serialize, Deserialize, JsonSchema, Debug)]
/// A representation of a Player that provides a view of the Player and the Turns made by that Player.
pub(crate) struct PlayerWithTurns {
    /// The name of the Player (which is kept current)
    pub(crate) name: String,
    /// The current Team of the player
    pub(crate) team: Option<TeamWithColors>,
    /// [DEPRECATED] The `platform` from which the user connects
    /// This will be removed in a later version of RR.
    // TODO: Deprecate
    pub(crate) platform: String,
    pub(crate) ratings: Ratings,
    pub(crate) stats: Stats,
    pub(crate) turns: Vec<PastTurn>,
    pub(crate) is_alt: bool,
}

#[derive(Queryable, Serialize, Deserialize, JsonSchema, Debug)]
pub(crate) struct PlayerInTurns {
    pub(crate) team: String,
    pub(crate) player: String,
    pub(crate) stars: i32,
    pub(crate) weight: f64,
    pub(crate) multiplier: f64,
    pub(crate) mvp: bool,
    pub(crate) power: f64,
}

#[derive(Queryable, Serialize, Deserialize, JsonSchema, Debug)]
pub struct PlayerWithTurnsAndAdditionalTeam {
    pub name: String,
    pub team: Option<TeamWithColors>,
    pub active_team: Option<TeamWithColors>,
    /// [DEPRECATED] The `platform` from which the user connects
    /// This will be removed in a later version of RR.
    // TODO: Deprecate
    pub platform: String,
    pub ratings: Ratings,
    pub stats: Stats,
    pub turns: Vec<PastTurn>,
    pub award: Vec<Award>,
    pub is_alt: bool,
}

#[derive(Queryable, Serialize, Deserialize, JsonSchema)]
pub(crate) struct PlayerSummary {
    pub(crate) name: String,
    /// [DEPRECATED] The `platform` from which the user connects
    /// This will be removed in a later version of RR.
    // TODO: Deprecate
    pub(crate) platform: String,
    pub(crate) team: Option<String>,
}

impl PlayerSummary {
    pub(crate) fn load(
        conn: &mut PgConnection,
    ) -> Result<Vec<PlayerSummary>, diesel::result::Error> {
        user::table
            .left_join(team::table.on(team::id.eq(user::playing_for)))
            .select((user::uname, user::platform, team::tname.nullable()))
            .load::<PlayerSummary>(conn)
    }
}

impl PlayerWithTurnsAndAdditionalTeam {
    pub(crate) fn load(
        name: Vec<String>,
        team_assigned: bool,
        conn: &mut PgConnection,
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
                let ciName: Vec<String> = name;
                let award: Vec<Award> = award::table
                    .inner_join(award_info::table)
                    .inner_join(user::table)
                    .filter(user::uname.eq(&me[0].name))
                    .select((award_info::name, award_info::info))
                    .load(conn)
                    .unwrap_or_default();
                let results = user::table
                    .filter(user::uname.eq_any(ciName))
                    .filter(not(user::current_team.eq(status_code)))
                    .left_join(team::table.on(team::id.eq(user::playing_for)))
                    .select((
                        team::tname.nullable(),
                        team::color_1.nullable(),
                        team::color_2.nullable(),
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
                        award,
                        is_alt: me[0].is_alt,
                    }),
                    Err(_e) => Some(PlayerWithTurnsAndAdditionalTeam {
                        name: me[0].name.clone(),
                        team: None,
                        active_team: None,
                        platform: me[0].platform.clone(),
                        ratings: me[0].ratings.clone(),
                        stats: me[0].stats.clone(),
                        turns: me[0].turns.clone(),
                        award,
                        is_alt: me[0].is_alt,
                    }),
                }
            }
            _ => None,
        }
    }
    pub(crate) fn load_all(
        names: Vec<String>,
        team_assigned: bool,
        conn: &mut PgConnection,
    ) -> Vec<PlayerWithTurnsAndAdditionalTeam> {
        let mut ret: Vec<PlayerWithTurnsAndAdditionalTeam> = vec![];
        let me = PlayerWithTurns::load(names, true, conn);
        for user in me {
            use diesel::dsl::not;
            let status_code: i32 = match team_assigned {
                true => 0,
                false => -1,
            };
            let ciName: String = user.name.clone();
            let award: Vec<Award> = award::table
                .inner_join(award_info::table)
                .inner_join(user::table)
                .filter(user::uname.eq(&user.name))
                .select((award_info::name, award_info::info))
                .load(conn)
                .unwrap_or_default();
            let results = user::table
                .filter(user::uname.eq(ciName))
                .filter(not(user::current_team.eq(status_code)))
                .left_join(team::table.on(team::id.eq(user::playing_for)))
                .select((
                    team::tname.nullable(),
                    team::color_1.nullable(),
                    team::color_2.nullable(),
                ))
                .first::<Team>(conn);
            match results {
                Ok(results) => {
                    let u = PlayerWithTurnsAndAdditionalTeam {
                        name: user.name.clone(),
                        team: user.team.clone(),
                        active_team: Some(TeamWithColors {
                            name: results.name,
                            colors: Colors {
                                primary: results.color_1.unwrap_or_else(|| String::from("#000")),
                                secondary: results.color_2.unwrap_or_else(|| String::from("#000")),
                            },
                        }),
                        platform: user.platform.clone(),
                        ratings: user.ratings.clone(),
                        stats: user.stats.clone(),
                        turns: user.turns.clone(),
                        award,
                        is_alt: user.is_alt,
                    };
                    ret.push(u)
                }
                Err(_e) => {
                    let u = PlayerWithTurnsAndAdditionalTeam {
                        name: user.name.clone(),
                        team: None,
                        active_team: None,
                        platform: user.platform.clone(),
                        ratings: user.ratings.clone(),
                        stats: user.stats.clone(),
                        turns: user.turns.clone(),
                        award,
                        is_alt: user.is_alt,
                    };
                    ret.push(u)
                }
            };
        }
        ret
    }
}

impl PlayerWithTurns {
    pub(crate) fn load(
        name: Vec<String>,
        team_assigned: bool,
        conn: &mut PgConnection,
    ) -> Vec<PlayerWithTurns> {
        use diesel::dsl::not;
        let status_code: i32 = match team_assigned {
            true => 0,
            false => -1,
        };
        let ciName: Vec<String> = name;
        let results = user::table
            .filter(user::uname.eq_any(ciName))
            .filter(not(user::current_team.eq(status_code)))
            .left_join(team::table.on(team::id.eq(user::current_team)))
            .select((
                (
                    user::id,
                    user::uname,
                    user::platform,
                    user::turns,
                    user::game_turns,
                    user::mvps,
                    user::streak,
                    user::is_alt,
                    //user::award,
                ),
                (
                    team::tname.nullable(),
                    team::color_1.nullable(),
                    team::color_2.nullable(),
                ),
            ))
            .load::<(User, Team)>(conn)
            .expect("Error loading user");
        let mut out = Vec::new();
        for user in results {
            let stats = Stats {
                totalTurns: user.0.turns.unwrap_or(0),
                gameTurns: user.0.game_turns.unwrap_or(0),
                mvps: user.0.mvps.unwrap_or(0),
                streak: user.0.streak.unwrap_or(0),
                //award: user.0.award.unwrap_or(0),
            };
            let user_turns = turn::table
                .filter(turn::user_id.eq(&user.0.id))
                .inner_join(team::table.on(team::id.eq(turn::team)))
                .inner_join(territory::table.on(territory::id.eq(turn::territory)))
                .inner_join(turn::table.on(turn::id.eq(turn::turn_id)))
                .select((
                    turn::season,
                    turn::day,
                    turn::stars,
                    turn::mvp,
                    territory::name,
                    team::tname,
                    turn::weight,
                    turn::multiplier,
                    turn::power,
                ))
                .order(turn::turn_id.desc())
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
                turns: user_turns,
                is_alt: user.0.is_alt,
            };
            out.push(uwp);
        }
        out
    }
}

impl TeamPlayer {
    pub(crate) fn load(
        tname: Vec<String>,
        conn: &mut PgConnection,
    ) -> Result<Vec<TeamPlayer>, diesel::result::Error> {
        let ciTname: Vec<String> = tname;
        move::table
            .filter(move::tname.eq_any(ciTname))
            .select((
                move::tname,
                move::uname,
                move::turns,
                move::mvps,
                (move::season, move::day, move::stars),
            ))
            .load::<TeamPlayer>(conn)
    }

    pub(crate) fn loadall(
        conn: &mut PgConnection,
    ) -> Result<Vec<TeamPlayer>, diesel::result::Error> {
        move::table
            .select((
                move::tname,
                move::uname,
                move::turns,
                move::mvps,
                (move::season, move::day, move::stars),
            ))
            .load::<TeamPlayer>(conn)
    }
}

impl TeamMerc {
    pub(crate) fn load_mercs(
        tname: Vec<String>,
        conn: &mut PgConnection,
    ) -> Result<Vec<TeamMerc>, diesel::result::Error> {
        let ciTname: Vec<String> = tname;
        allow_tables_to_appear_in_same_query!(user, move);
        let teamIds = team::table
            .filter(team::tname.eq_any(ciTname))
            .select(team::id)
            .load::<i32>(conn)?;
        use diesel::dsl::not;
        user::table
            .inner_join(team::table.on(team::id.eq(user::current_team)))
            .filter(user::playing_for.eq_any(teamIds))
            .filter(not(user::playing_for.eq(user::current_team)))
            .select((
                team::tname,
                user::uname,
                user::turns,
                user::mvps,
                user::overall,
            ))
            .load::<TeamMerc>(conn)
    }
}

impl PlayerInTurns {
    pub(crate) fn load(
        season: &i32,
        day: &i32,
        territory: &str,
        conn: &mut PgConnection,
    ) -> Result<Vec<PlayerInTurns>, Error> {
        let ciTerritory = territory.to_owned();
        dbg!(&season, &day, &ciTerritory);
        turn::table
            .inner_join(territory::table.on(turn::territory.eq(territory::id)))
            .inner_join(team::table.on(turn::team.eq(team::id)))
            .inner_join(turn::table.on(turn::turn_id.eq(turn::id)))
            .inner_join(user::table.on(turn::user_id.eq(user::id)))
            .select((
                team::tname,
                user::uname,
                turn::stars,
                turn::weight,
                turn::multiplier,
                turn::mvp,
                turn::power,
            ))
            .filter(turn::day.eq(day))
            .filter(turn::season.eq(season))
            .filter(territory::name.eq(ciTerritory))
            .load::<PlayerInTurns>(conn)
    }
}

impl UserId for User {
    fn id(&self) -> i32 {
        self.id
    }
}
impl User {
    pub fn load(name: String, platform: String, conn: &mut PgConnection) -> Result<User, Error> {
        user::table
            .filter(user::uname.eq(name))
            .filter(user::platform.eq(platform))
            .select((
                user::id,
                user::uname,
                user::platform,
                user::turns,
                user::game_turns,
                user::mvps,
                user::streak,
                user::is_alt, //user::award,
            ))
            .first::<User>(conn)
    }

    pub fn search(s: String, limit: i32, conn: &mut PgConnection) -> Result<Vec<String>, Error> {
        user::table
            .filter(user::uname.ilike(s))
            .select(user::uname)
            .limit(limit.into())
            .load::<String>(conn)
    }
}
