/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use crate::model::team::TeamWithColors;
use crate::model::turn::{LastTurn, PastTurn};
use crate::model::{Colors, Ratings, Stats, Team, Turn, UserId};
use crate::schema::{award_info, awards, moves, past_turns, teams, territories, turninfo, users};
use diesel::prelude::*;
use diesel::result::Error;

use schemars::JsonSchema;

#[derive(Queryable, Serialize, Deserialize, JsonSchema, Debug)]
/// An official distinction applied by the game moderators to distinguish
/// users for a given reson/purpose. Awards do not affect actual gameplay.
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
    pub(crate) is_alt: bool, //pub(crate) awards: Option<i32>, //    pub team: Option<String>
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
    pub awards: Vec<Award>,
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
                let awards: Vec<Award> = awards::table
                    .inner_join(award_info::table)
                    .inner_join(users::table)
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
                        awards,
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
            let awards: Vec<Award> = awards::table
                .inner_join(award_info::table)
                .inner_join(users::table)
                .filter(users::uname.eq(&user.name))
                .select((award_info::name, award_info::info))
                .load(conn)
                .unwrap_or_default();
            let results = users::table
                .filter(users::uname.eq(ciName))
                .filter(not(users::current_team.eq(status_code)))
                .left_join(teams::table.on(teams::id.eq(users::playing_for)))
                .select((
                    teams::tname.nullable(),
                    teams::color_1.nullable(),
                    teams::color_2.nullable(),
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
                        awards,
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
                        awards,
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
                    users::is_alt,
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
                    past_turns::weight,
                    past_turns::multiplier,
                    past_turns::power,
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

    pub(crate) fn loadall(
        conn: &mut PgConnection,
    ) -> Result<Vec<TeamPlayer>, diesel::result::Error> {
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
        conn: &mut PgConnection,
    ) -> Result<Vec<TeamMerc>, diesel::result::Error> {
        let ciTname: Vec<String> = tname;
        allow_tables_to_appear_in_same_query!(users, moves);
        let teamIds = teams::table
            .filter(teams::tname.eq_any(ciTname))
            .select(teams::id)
            .load::<i32>(conn)?;
        use diesel::dsl::not;
        users::table
            .inner_join(teams::table.on(teams::id.eq(users::current_team)))
            .filter(users::playing_for.eq_any(teamIds))
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
        conn: &mut PgConnection,
    ) -> Result<Vec<PlayerInTurns>, Error> {
        let ciTerritory = territory.to_owned();
        dbg!(&season, &day, &ciTerritory);
        past_turns::table
            .inner_join(territories::table.on(past_turns::territory.eq(territories::id)))
            .inner_join(teams::table.on(past_turns::team.eq(teams::id)))
            .inner_join(turninfo::table.on(past_turns::turn_id.eq(turninfo::id)))
            .inner_join(users::table.on(past_turns::user_id.eq(users::id)))
            .select((
                teams::tname,
                users::uname,
                past_turns::stars,
                past_turns::weight,
                past_turns::multiplier,
                past_turns::mvp,
                past_turns::power,
            ))
            .filter(turninfo::day.eq(day))
            .filter(turninfo::season.eq(season))
            .filter(territories::name.eq(ciTerritory))
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
        users::table
            .filter(users::uname.eq(name))
            .filter(users::platform.eq(platform))
            .select((
                users::id,
                users::uname,
                users::platform,
                users::turns,
                users::game_turns,
                users::mvps,
                users::streak,
                users::is_alt, //users::awards,
            ))
            .first::<User>(conn)
    }

    pub fn search(s: String, limit: i32, conn: &mut PgConnection) -> Result<Vec<String>, Error> {
        users::table
            .filter(users::uname.ilike(s))
            .select(users::uname)
            .limit(limit.into())
            .load::<String>(conn)
    }
}
