use crate::model::*;
use crate::schema::*;
use diesel::prelude::*;
use diesel::result::Error;
#[derive(Queryable, Serialize, Deserialize, Associations)]
#[table_name = "teams"]
pub struct Team {
    pub name: Option<String>,
    pub color_1: Option<String>,
    pub color_2: Option<String>,
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TeamWithColors {
    pub name: Option<String>,
    pub colors: Colors,
}
#[derive(Queryable, Serialize, Deserialize, Clone, Debug)]
pub struct Colors {
    pub primary: Option<String>,
    pub secondary: Option<String>,
}
#[derive(Queryable, Serialize, Deserialize)]
pub struct TeamInfo {
    pub id: i32,
    pub name: Option<String>,
    pub logo: Option<String>,
    pub colors: Colors,
}

#[derive(Queryable, Serialize, Deserialize)]
pub struct TeamInTurns {
    pub team: String,
    pub color: String,
    pub secondaryColor: String,
    pub players: i32,
    pub power: f64,
    pub chance: f64,
    pub breakdown: StarBreakdown64,
}

#[derive(Queryable, Serialize, Deserialize, Associations)]
#[table_name = "team_player_moves"]
pub struct TeamPlayerMoves {
    pub id: i32,
    pub season: Option<i32>,
    pub day: Option<i32>,
    pub team: Option<String>,
    pub player: Option<String>,
    pub stars: Option<i32>,
    pub mvp: Option<bool>,
    pub territory: Option<String>,
    pub regularTeam: Option<String>,
}

impl TeamInfo {
    pub fn load(conn: &PgConnection) -> Vec<TeamInfo> {
        teams::table
            .select((teams::id, teams::tname, teams::logo, (teams::color_1, teams::color_2)))
            .load::<TeamInfo>(conn)
            .expect("Error loading teams")
    }
}

impl TeamPlayerMoves {
    pub fn load(
        season_seek: i32,
        day_seek: i32,
        team: Option<String>,
        conn: &PgConnection,
    ) -> Vec<TeamPlayerMoves> {
        match team {
            Some(team_seek) => {
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
                    .filter(team_player_moves::team.eq(team_seek))
                    .load::<TeamPlayerMoves>(conn)
                    .expect("Error loading moves")
            }
            None => {
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
                    .load::<TeamPlayerMoves>(conn)
                    .expect("Error loading moves")
            }
        }
    }
}

impl TeamInTurns {
    pub fn load(
        season: &i32,
        day: &i32,
        territory: &str,
        conn: &PgConnection,
    ) -> Result<Vec<TeamInTurns>, Error> {
        odds::table
            .select((
                odds::team_name,
                odds::color,
                odds::secondary_color,
                odds::players,
                odds::teampower,
                odds::chance,
                (odds::ones, odds::twos, odds::threes, odds::fours, odds::fives),
            ))
            .filter(odds::day.eq(day))
            .filter(odds::season.eq(season))
            .filter(odds::territory_name.eq(territory))
            .load::<TeamInTurns>(conn)
    }
}
