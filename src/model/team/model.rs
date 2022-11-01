/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use crate::model::StarBreakdown64;
use crate::schema::{odds, team_player_moves, teams};
use diesel::prelude::*;
use diesel_citext::types::CiString;
use schemars::JsonSchema;

#[derive(Queryable, Serialize, Deserialize, JsonSchema, Associations)]
#[table_name = "teams"]
pub(crate) struct Team {
    pub(crate) name: Option<String>,
    pub(crate) color_1: Option<String>,
    pub(crate) color_2: Option<String>,
}
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub(crate) struct TeamWithColors {
    pub(crate) name: Option<String>,
    pub(crate) colors: Colors,
}
#[derive(Queryable, Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub(crate) struct Colors {
    pub(crate) primary: String,
    pub(crate) secondary: String,
}
#[derive(Queryable, Serialize, Deserialize, JsonSchema)]
pub(crate) struct TeamInfo {
    pub(crate) id: i32,
    pub(crate) name: String,
    pub(crate) logo: Option<String>,
    pub(crate) colors: Colors,
    pub(crate) seasons: Vec<i32>,
}

#[derive(Queryable, Serialize, Deserialize, JsonSchema, Debug)]
pub(crate) struct TeamInTurns {
    pub(crate) team: CiString,
    pub(crate) color: String,
    pub(crate) secondaryColor: String,
    pub(crate) players: i32,
    pub(crate) power: f64,
    pub(crate) chance: f64,
    pub(crate) breakdown: StarBreakdown64,
}

#[derive(Queryable, Serialize, Deserialize, JsonSchema, Associations)]
#[table_name = "team_player_moves"]
pub(crate) struct TeamPlayerMoves {
    pub(crate) id: i32,
    pub(crate) season: Option<i32>,
    pub(crate) day: Option<i32>,
    pub(crate) team: Option<String>,
    pub(crate) player: Option<String>,
    pub(crate) stars: Option<i32>,
    pub(crate) mvp: Option<bool>,
    pub(crate) territory: Option<String>,
    pub(crate) regularTeam: Option<String>,
}

impl TeamInfo {
    pub(crate) fn load(conn: &PgConnection) -> Vec<TeamInfo> {
        teams::table
            .select((
                teams::id,
                teams::tname,
                teams::logo,
                (teams::color_1, teams::color_2),
                teams::seasons,
            ))
            .load::<TeamInfo>(conn)
            .expect("Error loading teams")
    }
}

impl Default for TeamWithColors {
    fn default() -> TeamWithColors {
        TeamWithColors {
            name: None,
            colors: Colors {
                primary: String::from("#000"),
                secondary: String::from("#000"),
            },
        }
    }
}

impl TeamPlayerMoves {
    pub(crate) fn load(
        season_seek: i32,
        day_seek: i32,
        team: Option<String>,
        conn: &PgConnection,
    ) -> Result<Vec<TeamPlayerMoves>, diesel::result::Error> {
        match team {
            Some(team_seek) => {
                let ciTeam_seek = CiString::from(team_seek);
                team_player_moves::table
                    .select((
                        team_player_moves::id,
                        team_player_moves::season,
                        team_player_moves::day,
                        team_player_moves::team,
                        team_player_moves::player,
                        team_player_moves::stars,
                        team_player_moves::mvp,
                        team_player_moves::territory,
                        team_player_moves::regularteam,
                    ))
                    .filter(team_player_moves::season.eq(season_seek))
                    .filter(team_player_moves::day.eq(day_seek))
                    .filter(team_player_moves::team.eq(ciTeam_seek))
                    .load::<TeamPlayerMoves>(conn)
            }
            None => team_player_moves::table
                .select((
                    team_player_moves::id,
                    team_player_moves::season,
                    team_player_moves::day,
                    team_player_moves::team,
                    team_player_moves::player,
                    team_player_moves::stars,
                    team_player_moves::mvp,
                    team_player_moves::territory,
                    team_player_moves::regularteam,
                ))
                .filter(team_player_moves::season.eq(season_seek))
                .filter(team_player_moves::day.eq(day_seek))
                .load::<TeamPlayerMoves>(conn),
        }
    }
}

impl TeamInTurns {
    pub(crate) fn load(
        season: &i32,
        day: &i32,
        territory: &str,
        conn: &PgConnection,
    ) -> Result<Vec<TeamInTurns>, diesel::result::Error> {
        odds::table
            .select((
                odds::team_name,
                odds::color,
                odds::secondary_color,
                odds::players,
                odds::teampower,
                odds::chance,
                (
                    odds::ones,
                    odds::twos,
                    odds::threes,
                    odds::fours,
                    odds::fives,
                ),
            ))
            .filter(odds::day.eq(day))
            .filter(odds::season.eq(season))
            .filter(odds::territory_name.eq(CiString::from(territory)))
            .load::<TeamInTurns>(conn)
    }
}
