use crate::model::team::TeamWithColors;
use crate::model::turn::{LastTurn, PastTurn};
use crate::model::{Colors, Ratings, Stats, Team, Turn};
use crate::schema::*;
use diesel::prelude::*;
use diesel::result::Error;
use diesel_citext::types::CiString;

#[derive(Serialize)]
pub struct Player {
    pub id: i32,
    pub name: CiString,
    pub team: Team,
    pub ratings: Ratings,
    pub stats: Stats,
    pub turns: Vec<Turn>,
}

#[derive(Queryable, Serialize, Deserialize)]
pub struct TeamPlayer {
    pub team: Option<CiString>,
    pub player: Option<CiString>,
    pub turnsPlayed: Option<i32>,
    pub mvps: Option<i32>,
    pub lastTurn: LastTurn,
}
#[derive(Queryable, Identifiable, Associations, Serialize, Deserialize)]
pub struct User {
    pub id: i32,
    pub uname: CiString,
    pub platform: CiString,
    pub turns: Option<i32>,
    pub game_turns: Option<i32>,
    pub mvps: Option<i32>,
    pub streak: Option<i32>,
    pub awards: Option<i32>, //    pub team: Option<String>
}

#[derive(Queryable, Serialize, Deserialize)]
pub struct PlayerWithTurns {
    pub name: CiString,
    pub team: Option<TeamWithColors>,
    pub platform: CiString,
    pub ratings: Ratings,
    pub stats: Stats,
    pub turns: Vec<PastTurn>,
}

#[derive(Queryable, Serialize, Deserialize)]
pub struct PlayerInTurns {
    pub team: Option<CiString>,
    pub player: Option<CiString>,
    pub stars: Option<i32>,
    pub weight: i32,
    pub multiplier: f64,
    pub mvp: Option<bool>,
    pub power: f64,
}

#[derive(Queryable, Serialize, Deserialize)]
pub struct PlayerWithTurnsAndAdditionalTeam {
    pub name: CiString,
    pub team: Option<TeamWithColors>,
    pub active_team: Option<TeamWithColors>,
    pub platform: CiString,
    pub ratings: Ratings,
    pub stats: Stats,
    pub turns: Vec<PastTurn>,
}

impl PlayerWithTurnsAndAdditionalTeam {
    pub fn load(
        name: Vec<String>,
        team_assigned: bool,
        conn: &PgConnection,
    ) -> Option<PlayerWithTurnsAndAdditionalTeam> {
        let me = PlayerWithTurns::load(name.clone(), true, &conn);
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
                    Ok(results) => {
                        Some(PlayerWithTurnsAndAdditionalTeam {
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
                        })
                    }
                    Err(_e) => {
                        Some(PlayerWithTurnsAndAdditionalTeam {
                            name: me[0].name.clone(),
                            team: None,
                            active_team: None,
                            platform: me[0].platform.clone(),
                            ratings: me[0].ratings.clone(),
                            stats: me[0].stats.clone(),
                            turns: me[0].turns.clone(),
                        })
                    }
                }
            }
            _ => None,
        }
    }
}

impl PlayerWithTurns {
    pub fn load(
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
                    users::awards,
                ),
                (teams::tname.nullable(), teams::color_1.nullable(), teams::color_2.nullable()),
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
                awards: user.0.awards.unwrap_or(0),
            };
            let users_turns = past_turns::table
                .filter(past_turns::user_id.eq(&user.0.id))
                .inner_join(teams::table.on(teams::id.eq(past_turns::team)))
                .inner_join(territories::table.on(territories::id.eq(past_turns::territory)))
                .select((
                    past_turns::season,
                    past_turns::day,
                    past_turns::stars,
                    past_turns::mvp,
                    territories::name,
                    teams::tname,
                ))
                .order((past_turns::season.desc(), past_turns::day.desc()))
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
    pub fn load(tname: Vec<String>, conn: &PgConnection) -> Vec<TeamPlayer> {
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
            .expect("Error loading players")
    }

    pub fn loadall(conn: &PgConnection) -> Vec<TeamPlayer> {
        moves::table
            .select((
                moves::tname,
                moves::uname,
                moves::turns,
                moves::mvps,
                (moves::season, moves::day, moves::stars),
            ))
            .load::<TeamPlayer>(conn)
            .expect("Error loading players")
    }
}

impl PlayerInTurns {
    pub fn load(
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
                users::awards,
            ))
            .first::<User>(conn)
    }
}
