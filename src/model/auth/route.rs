/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use crate::db::DbConn;
use crate::model::reddit::route::{audit_trail, Cip, UA};
#[cfg(feature = "risk_captcha")]
use crate::model::{
    Claims, CurrentStrength, Latest, Log, MoveInfo, MoveSub, PlayerWithTurnsAndAdditionalTeam,
    Poll, PollResponse, Ratings, Stats, TurnInfo, UpdateUser, UserIdFast,
};
use crate::schema::{
    cfbr_stats, region_ownership, territory_adjacency, territory_ownership, turns, users,
};
use crate::sys::SysInfo;
use diesel::prelude::*;
use diesel::result::Error;
use rocket::http::{CookieJar, Status};
use rocket::State;
extern crate rand;
use diesel_citext::types::CiString;
use rand::{thread_rng, Rng};
use rocket::serde::json::Json;
use rocket_recaptcha_v3::{ReCaptcha, ReCaptchaToken, V2};
use serde_json::json;

#[derive(Serialize, Deserialize, Debug)]
pub struct StatusWrapper {
    code: i32,
    message: String,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum EitherPorS {
    PlayerWithTurnsAndAdditionalTeam(Box<PlayerWithTurnsAndAdditionalTeam>),
    StatusWrapper(StatusWrapper),
    String(std::string::String),
}

/// # Me
/// Retrieves all information about currently logged-in user. Should not be accessed by any
/// scraping programs.
#[openapi(skip)]
#[get("/me")]
pub(crate) async fn me(
    cookies: &CookieJar<'_>,
    conn: DbConn,
    config: &State<SysInfo>,
) -> Result<Json<impl serde::ser::Serialize>, crate::Error> {
    let c = match Claims::from_private_cookie(cookies, config) {
        Ok(c) => c,
        Err(_) => {
            return Ok(Json(EitherPorS::StatusWrapper(StatusWrapper {
                code: 4000,
                message: "Unauthenticated".to_owned(),
            })));
        }
    };
    let username = c.0.user.clone();
    let user = conn
        .run(move |connection| {
            PlayerWithTurnsAndAdditionalTeam::load(vec![username], false, connection)
        })
        .await
        .ok_or(crate::Error::NotFound {})?;
    if user.name.to_lowercase() == c.0.user.to_lowercase() {
        std::result::Result::Ok(Json(EitherPorS::PlayerWithTurnsAndAdditionalTeam(
            Box::new(user),
        )))
    } else {
        std::result::Result::Err(crate::Error::NotFound {})
    }
}

#[get("/join?<team>", rank = 1)]
pub(crate) async fn join_team(
    team: i32,
    cookies: &CookieJar<'_>,
    conn: DbConn,
    config: &State<SysInfo>,
) -> Result<Json<String>, crate::Error> {
    // Do not allow joining Unjonable Placeholder
    if team <= 0 {
        return Err(crate::Error::BadRequest {});
    }
    // Get user information from cookies
    let c = Claims::from_private_cookie(cookies, config)?;
    //see if user already has team, and if user has current_team
    let username = c.0.user.clone();
    let users = conn
        .run(move |connection| {
            PlayerWithTurnsAndAdditionalTeam::load(vec![username], false, connection)
        })
        .await
        .ok_or(crate::Error::Unauthorized {})?;

    // Check that DB and cookie correspond, if not, yeet!
    if users.name.to_lowercase() != c.0.user.to_lowercase() {
        return std::result::Result::Err(crate::Error::Unauthorized {});
    }

    // Does the team they want to join have territories?
    // check that team has territories
    let has_territories: bool = conn
        .run(move |connection| CurrentStrength::load_id(team, connection))
        .await
        .map_err(|_| crate::Error::BadRequest {})?
        .territories
        > 0;
    // If user has no team (and thus no active_team), then allow them to join anything
    if users.active_team.unwrap_or_default().name.is_some() {
        return std::result::Result::Err(crate::Error::BadRequest {});
    }

    // If user just needs new active team, we can do this
    if users.team.unwrap_or_default().name.is_some() {
        if has_territories {
            conn.run(move |cn| update_user(true, c.0.id, team, cn))
                .await?; //playing_for
            std::result::Result::Ok(Json(String::from("Okay")))
        } else {
            std::result::Result::Err(crate::Error::BadRequest {})
        }
    } else {
        // User needs BOTH team and active team. IF
        if has_territories {
            conn.run(move |cn| update_user(false, c.0.id, team, cn))
                .await?; //playing_for
            conn.run(move |cn| update_user(true, c.0.id, team, cn))
                .await?; //current_team
            std::result::Result::Ok(Json(String::from("Okay")))
        } else {
            conn.run(move |cn| update_user(false, c.0.id, team, cn))
                .await?; //current_team
            std::result::Result::Ok(Json(String::from("Partial")))
        }
    }
}

#[post("/my_move", rank = 1)]
pub(crate) async fn my_move(
    cookies: &CookieJar<'_>,
    conn: DbConn,
    config: &State<SysInfo>,
) -> Result<Json<EitherPorS>, crate::Error> {
    // Get latest turn
    let latest = conn
        .run(move |c| Latest::latest(c))
        .await
        .map_err(|_| crate::Error::InternalServerError {})?;
    // Get user information from cookies
    let c = match Claims::from_private_cookie(cookies, config) {
        Ok(c) => c,
        Err(_) => {
            return Ok(Json(EitherPorS::StatusWrapper(StatusWrapper {
                code: 4000,
                message: "Unauthenticated".to_owned(),
            })))
        }
    };
    // Return the territory the user has attacked
    std::result::Result::Ok(Json(EitherPorS::String(
        conn.run(move |connection| MoveInfo::get(latest.season, latest.day, c.0.id, connection))
            .await
            .territory
            .unwrap_or_else(|| String::from("")),
    )))
}

#[post("/move", rank = 1, format = "application/json", data = "<movesub>")]
pub(crate) async fn make_move<'v>(
    movesub: Json<MoveSub>,
    cookies: &CookieJar<'_>,
    cip: Cip,
    ua: UA,
    conn: DbConn,
    config: &State<SysInfo>,
    recaptcha: &State<ReCaptcha>,
    recaptcha_v2: &State<ReCaptcha<V2>>,
) -> Result<Json<StatusWrapper>, crate::Error> {
    let target = movesub.target;
    let rv: String = match movesub.token.as_ref() {
        Some(e) => format!("token={}", e),
        None => return Err(crate::Error::BadRequest {}),
    };
    let r = rocket::form::ValueField::parse(&rv);
    use crate::rocket::form::FromFormField;
    let recaptcha_token = ReCaptchaToken::from_value(r)
        .as_ref()
        .map_err(|e| {
            dbg!(e);
            crate::Error::BadRequest {}
        })?
        .clone();
    let recaptcha_return = recaptcha
        .verify(&recaptcha_token, None)
        .await
        .map_err(|e| {
            dbg!(e);
            crate::Error::InternalServerError {}
        })?;
    if recaptcha_return.action != Some("submit".to_string()) {
        return Err(crate::Error::BadRequest {});
    }

    let mut log = Log::begin(String::from("move"), target.to_string());

    // Get latest turn
    let latest = conn.run(move |c| TurnInfo::latest(c)).await.map_err(|_| {
        dbg!("Failed at point 1");
        crate::Error::InternalServerError {}
    })?;

    log.payload.push_str(&format!("Latest: {}\n", latest.id));

    // Get user information from cookies
    let c = Claims::from_private_cookie(cookies, config)?;

    log.payload.push_str(&format!("Claims: {:?}\n", c.0.id));

    let tmplatest = latest.clone();

    //get user's team information, and whether they can make that move
    let temp_pfix = c.0.clone();
    let msaon = movesub.aon;
    let (user, multiplier) = conn
        .run(move |connection| {
            handle_territory_info(&temp_pfix, target, &tmplatest, connection, msaon)
        })
        .await
        .map_err(|_| {
            dbg!("Failed at point 3");
            crate::Error::BadRequest {}
        })?;

    log.payload.push_str(&format!("User: {user:?}\n"));
    let uidfast = UserIdFast { id: user.0 };
    audit_trail(&uidfast, &json!(null), &cip.0, &ua.0, 2, &conn)
        .await
        .map_err(|_| {
            dbg!("Failed at point 3.1");
            crate::Error::BadRequest {}
        })?;

    //at this point we know the user is authorized to make the action, so let's go ahead and make it
    let user_stats = Stats {
        totalTurns: user.3.unwrap_or(0),
        gameTurns: user.4.unwrap_or(0),
        mvps: user.5.unwrap_or(0),
        streak: user.6.unwrap_or(0),
    };

    let user_ratings = Ratings::load(&user_stats);

    let user_weight: f64 = match user_ratings.overall {
        1 => 1.0,
        2 => 2.0,
        3 => 3.0,
        4 => 4.0,
        5 => 5.0,
        _ => 1.0,
    };
    let user_power: f64 = multiplier * user_weight;
    let mut merc: bool = false;

    if user.0 != user.7 {
        merc = true;
    }

    #[cfg(feature = "risk_captcha")]
    // User must complete captcha.
    if user.9 || recaptcha_return.score < 0.5 {
        // Check Captcha V2
        //dbg!(&movesub.token_v2);
        let v2_verif = match &movesub.token_v2 {
            None => false,
            Some(mv_tv2) => {
                if mv_tv2 == "" {
                    return std::result::Result::Ok(Json(StatusWrapper {
                code: 4004,
                message: "Captcha required.".to_string(),
            }));
                }
                let rv_v2: String = format!("token={}", mv_tv2);
                let r_v2 = rocket::form::ValueField::parse(&rv_v2);
                let recaptcha_token_v2 = ReCaptchaToken::from_value(r_v2)
                    .as_ref()
                    .map_err(|e| {
                        dbg!(e);
                        crate::Error::BadRequest {}
                    })?
                    .clone();
                let r_v2_result = recaptcha_v2
                    .verify(&recaptcha_token_v2, None)
                    .await
                    .map_err(|e| {
                        dbg!(e);
                        crate::Error::InternalServerError {}
                    })?;
                if r_v2_result.score > 0.5 {
                    true
                } else {
                    false
                }
            }
        };

        if !v2_verif {
            return std::result::Result::Ok(Json(StatusWrapper {
                code: 4004,
                message: "Captcha required.".to_string(),
            }));
        }
    }

    let insert_turn = conn
        .run(move |connection| {
            insert_turn(
                &user,
                user_ratings,
                &latest,
                target,
                multiplier,
                user_weight,
                user_power,
                merc,
                connection,
            )
        })
        .await
        .map_err(|_| {
            dbg!("Failed at point 4");
            crate::Error::BadRequest {}
        })?;

    if insert_turn.len() != 1 || insert_turn[0] != target {
        dbg!("Failed at point 5");
        return std::result::Result::Err(crate::Error::InternalServerError {});
    }

    log.payload
        .push_str(&format!("New Turn: {:?}\n", insert_turn[0]));

    //now we go update the user
    conn.run(move |connection| {
        UpdateUser::do_update(
            UpdateUser {
                id: user.1,
                overall: user_weight as i32,
                turns: user_stats.totalTurns,
                game_turns: user_stats.gameTurns,
                mvps: user_stats.mvps,
                streak: user_stats.streak,
                // awards: user_stats.awards,
            },
            connection,
        )
    })
    .await
    .map_err(|_| {
        dbg!("Failed at point 2");
        crate::Error::BadRequest {}
    })?;

    log.payload.push_str("User updated");

    conn.run(move |c| log.insert(c)).await?;

    // We got to the end!
    std::result::Result::Ok(Json(StatusWrapper {
        code: 2001,
        message: insert_turn[0].to_string(),
    }))
}

#[get("/polls", rank = 1)]
pub(crate) async fn get_polls(conn: DbConn) -> Result<Json<Vec<Poll>>, crate::Error> {
    // Get latest turn
    let latest = conn
        .run(move |c| Latest::latest(c))
        .await
        .map_err(|_| crate::Error::InternalServerError {})?;

    match conn
        .run(move |c| Poll::get(latest.season, latest.day, c))
        .await
    {
        Ok(polls) => std::result::Result::Ok(Json(polls)),
        Err(_E) => std::result::Result::Err(crate::Error::InternalServerError {}),
    }
}

#[get("/poll/respond?<poll>&<response>", rank = 1)]
pub(crate) async fn submit_poll(
    cookies: &CookieJar<'_>,
    conn: DbConn,
    config: &State<SysInfo>,
    poll: i32,
    response: bool,
) -> Result<Json<bool>, Status> {
    // get user id
    match cookies.get_private("jwt") {
        Some(cookie) => {
            match Claims::interpret(
                config.settings.cookie_key.as_bytes(),
                cookie.value().to_string(),
            ) {
                Ok(c) => {
                    // id, name Json(c.0.user)
                    match conn
                        .run(move |connection| {
                            PollResponse::upsert(
                                PollResponse {
                                    id: -1,
                                    poll,
                                    user_id: c.0.id,
                                    response,
                                },
                                connection,
                            )
                        })
                        .await
                    {
                        Ok(inner) => match inner {
                            1 => std::result::Result::Ok(Json(true)),
                            _ => std::result::Result::Err(Status::InternalServerError),
                        },
                        Err(_E) => std::result::Result::Err(Status::InternalServerError),
                    }
                }
                Err(_err) => std::result::Result::Err(Status::Unauthorized),
            }
        }
        None => std::result::Result::Err(Status::Unauthorized),
    }
}

#[get("/poll/response?<poll>", rank = 1)]
pub(crate) async fn view_response(
    cookies: &CookieJar<'_>,
    conn: DbConn,
    config: &State<SysInfo>,
    poll: i32,
) -> Result<Json<Vec<PollResponse>>, Status> {
    // get user id
    match cookies.get_private("jwt") {
        Some(cookie) => {
            match Claims::interpret(
                config.settings.cookie_key.as_bytes(),
                cookie.value().to_string(),
            ) {
                Ok(c) => {
                    // id, name Json(c.0.user)
                    match conn
                        .run(move |connection| PollResponse::get(poll, c.0.id, connection))
                        .await
                    {
                        Ok(responses) => std::result::Result::Ok(Json(responses)),
                        Err(_E) => std::result::Result::Err(Status::InternalServerError),
                    }
                }
                Err(_err) => std::result::Result::Err(Status::Unauthorized),
            }
        }
        None => std::result::Result::Err(Status::Unauthorized),
    }
}

// Returns the # of regions owned by `team` on `latest` turn.
pub(crate) fn handleregionalownership(
    latest: &TurnInfo,
    team: i32,
    conn: &PgConnection,
) -> QueryResult<i64> {
    use diesel::dsl::count;
    region_ownership::table
        .filter(region_ownership::season.eq(latest.season))
        .filter(region_ownership::day.eq(latest.day))
        .filter(region_ownership::owner_count.eq(1_i64))
        .filter(region_ownership::owners.contains(vec![team]))
        .select(count(region_ownership::owners))
        .first(conn)
}

pub(crate) fn get_regional_multiplier(mut territory_count: f64, team_id: i32) -> f64 {
    if team_id == 0 {
        return 1.0;
    }

    if team_id == 131 && cfg!(feature = "chaos") && territory_count >= 1.0 {
        territory_count -= 1.0;
    }

    territory_count *= 0.5;

    1.0 + territory_count
}

pub(crate) fn handle_territory_info(
    c: &Claims,
    target: i32,
    latest: &TurnInfo,
    conn: &PgConnection,
    aon: Option<bool>,
) -> Result<
    (
        (
            i32,
            i32,
            Option<i32>,
            Option<i32>,
            Option<i32>,
            Option<i32>,
            Option<i32>,
            // Option<i32>,
            i32,
            bool,
            bool,
        ),
        f64,
    ),
    String,
> {
    //get user now_playing team
    match users::table
        .filter(users::id.eq(c.id))
        .select((
            users::playing_for,
            users::id,
            users::overall,
            users::turns,
            users::game_turns,
            users::mvps,
            users::streak,
            // users::awards,
            users::current_team,
            users::is_alt,
            users::must_captcha,
        ))
        .first::<(
            i32,
            i32,
            Option<i32>,
            Option<i32>,
            Option<i32>,
            Option<i32>,
            Option<i32>,
            //Option<i32>,
            i32,
            bool,
            bool,
        )>(conn)
    {
        Ok(team_id) => match get_adjacent_territory_owners(target, latest, conn) {
            Ok(adjacent_territory_owners) => {
                match adjacent_territory_owners
                    .iter()
                    .position(|&x| x.0 == team_id.0)
                {
                    Some(_tuple_of_territory) => {
                        //dbg!(&adjacent_territory_owners);
                        let pos = adjacent_territory_owners
                            .iter()
                            .position(|&x| x.1 == target);
                        match adjacent_territory_owners
                            .iter()
                            .position(|&x| x.0 != team_id.0)
                        {
                            Some(_npos) => {
                                if team_id.0 != 0 {
                                    let regional_multiplier: f64 = get_regional_multiplier(
                                        handleregionalownership(latest, team_id.0, conn)
                                            .unwrap_or(0)
                                            as f64,
                                        team_id.0,
                                    );

                                    let mut aon_multiplier: i32 = 1;
                                    if aon == Some(true)
                                        && get_territory_number(team_id.0, latest, conn) == 1
                                        && latest.allOrNothingEnabled == Some(true)
                                    {
                                        let mut rng = thread_rng();
                                        // Triple or nothing
                                        aon_multiplier = 3 * rng.gen_range(0..2);
                                    }
                                    if adjacent_territory_owners[pos.unwrap()].0 == team_id.0 {
                                        Ok((
                                            team_id,
                                            1.5 * regional_multiplier * f64::from(aon_multiplier),
                                        ))
                                    } else {
                                        Ok((
                                            team_id,
                                            1.0 * regional_multiplier * f64::from(aon_multiplier),
                                        ))
                                    }
                                } else {
                                    let mut rng = thread_rng();
                                    let n: i32 = rng.gen_range(4..6);
                                    Ok((team_id, f64::from(n / 4)))
                                }
                            }
                            None => Err("You own all the surrounding territories".to_string()),
                        }
                    }
                    None => Err("You don't own that territory or an adjacent one".to_string()),
                }
            }
            Err(_er) => Err("You don't own that territory or an adjacent one".to_string()),
        },
        Err(_e) => Err("You don't own that territory or an adjacent one".to_string()),
    }
}

pub(crate) fn get_adjacent_territory_owners(
    target: i32,
    latest: &TurnInfo,
    conn: &PgConnection,
) -> Result<Vec<(i32, i32)>, Error> {
    territory_adjacency::table
        .filter(territory_adjacency::adjacent_id.eq(target))
        .filter(territory_adjacency::min_turn.lt(latest.id))
        .filter(territory_adjacency::max_turn.ge(latest.id))
        .filter(territory_ownership::turn_id.eq(latest.id))
        .inner_join(
            territory_ownership::table
                .on(territory_ownership::territory_id.eq(territory_adjacency::territory_id)),
        )
        .select((
            territory_ownership::owner_id,
            territory_ownership::territory_id,
        ))
        .load::<(i32, i32)>(conn)
}

pub(crate) fn get_territory_number(team: i32, latest: &TurnInfo, conn: &PgConnection) -> i32 {
    use diesel::dsl::count;
    territory_ownership::table
        .filter(territory_ownership::turn_id.eq(latest.id))
        .filter(territory_ownership::owner_id.eq(team))
        .select(count(territory_ownership::owner_id))
        .first(conn)
        .unwrap_or(0) as i32
}

#[allow(dead_code)]
pub(crate) fn get_cfb_points(name: String, conn: &PgConnection) -> i64 {
    match cfbr_stats::table
        .filter(cfbr_stats::player.eq(CiString::from(name)))
        .select(cfbr_stats::stars)
        .first(conn)
        .unwrap_or(1)
    {
        1 => 1,
        2 => 2,
        3 => 3,
        4 => 4,
        5 => 5,
        _ => 1,
    }
}

pub(crate) fn insert_turn(
    user: &(
        i32,
        i32,
        Option<i32>,
        Option<i32>,
        Option<i32>,
        Option<i32>,
        Option<i32>,
        //Option<i32>,
        i32,
        bool,
        bool,
    ),
    user_ratings: Ratings,
    latest: &TurnInfo,
    target: i32,
    multiplier: f64,
    user_weight: f64,
    user_power: f64,
    merc: bool,
    conn: &PgConnection,
) -> QueryResult<Vec<i32>> {
    let alt_score: i32 = match user.8 {
        true => 175,
        false => 0,
    };
    let power: f64 = match user.8 {
        true => 0.0,
        false => user_power,
    };
    diesel::insert_into(turns::table)
        .values((
            turns::user_id.eq(user.1),
            turns::turn_id.eq(latest.id),
            turns::territory.eq(target),
            turns::mvp.eq(false),
            turns::power.eq(power),
            turns::multiplier.eq(multiplier),
            turns::weight.eq(user_weight),
            turns::stars.eq(user_ratings.overall),
            turns::team.eq(user.0),
            turns::alt_score.eq(alt_score),
            turns::merc.eq(merc),
        ))
        .on_conflict((turns::user_id, turns::turn_id))
        .do_update()
        .set((
            turns::alt_score.eq(alt_score),
            turns::territory.eq(target),
            turns::power.eq(power),
            turns::multiplier.eq(multiplier),
        ))
        .returning(turns::territory)
        .get_results(conn)
}

pub(crate) fn update_user(
    new: bool,
    user: i32,
    team: i32,
    conn: &PgConnection,
) -> QueryResult<usize> {
    match new {
        false => diesel::update(users::table)
            .filter(users::id.eq(user))
            .set(users::current_team.eq(team))
            .execute(conn),
        true => diesel::update(users::table)
            .filter(users::id.eq(user))
            .set(users::playing_for.eq(team))
            .execute(conn),
    }
}
#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_get_regional_ncaa() {
        // Assumption: COUNT() never goes below 0.0
        assert_eq!(get_regional_multiplier(1.0, 0), 1.0); // Neither should NCAA
        assert_eq!(get_regional_multiplier(2.0, 0), 1.0); // Neither should NCAA
        assert_eq!(get_regional_multiplier(100.0, 0), 1.0); // Neither should NCAA
        assert_eq!(get_regional_multiplier(0.0, 0), 1.0); // Never go < 1.0
    }

    #[test]
    fn test_get_regional_chaos() {
        // Assumption: COUNT() never goes below 0.0
        if cfg!(feature = "chaos") {
            assert_eq!(get_regional_multiplier(1.0, 131), 1.0); // Chaos should not get additional for regions
            assert_eq!(get_regional_multiplier(2.0, 131), 1.5); // But Chaos should get some credit for > 1, unlike NCAA
            assert_eq!(get_regional_multiplier(0.0, 131), 1.0); // Never go < 1.0
        } else {
            assert_eq!(get_regional_multiplier(1.0, 131), 1.5); // Chaos behaves like a normal team
            assert_eq!(get_regional_multiplier(2.0, 131), 2.0); // Chaos behaves like a normal team
            assert_eq!(get_regional_multiplier(0.0, 131), 1.0); // Never go < 1.0
        }
    }

    #[test]
    fn test_get_regional_normal() {
        // Assumption: COUNT() never goes below 0.0
        assert_eq!(get_regional_multiplier(0.0, 11), 1.0); // Never go < 1.0
        assert_eq!(get_regional_multiplier(1.0, 11), 1.5); // Test `normal` case
        assert_eq!(get_regional_multiplier(2.0, 11), 2.0); // Test `normal` case
        assert_eq!(get_regional_multiplier(3.0, 11), 2.5); // Test `normal` case
    }
}
