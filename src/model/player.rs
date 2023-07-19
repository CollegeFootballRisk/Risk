/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use crate::db::DbConn;
use crate::error::{MapRre, Result};
use crate::model::Role;
use crate::schema;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use okapi::openapi3::SchemaObject;
use rocket::form::Form;
use rocket::serde::json::Json;
use rocket_okapi::JsonSchema;
use schemars::gen::SchemaGenerator;
use schemars::schema::InstanceType;
use uuid::Uuid;
/// # Lite Team
/// Simple rendition of a team, with minimal information
#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct SimpleTeam {
    id: i32,
    name: String,
}

/// # Lite Player
/// Simple rendition of a player, with minimal information
#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct SimplePlayer {
    /// [Limit: 36 Char] A player's id (UUID)
    id: Uuid,
    /// [Limit: 64 Char] A player's local username
    name: String,
    team: SimpleTeam,
}

/// # Player Metadata
/// Full metadata of a player
///
/// _**Note:** Because players may request that their account be deleted (e.g. it is merged into
/// another account or the user requests permanent deletion), a user may "disapper". If a user has
/// requested their account to be merged, this will appear in the Event log so that their ID can be
/// remapped accordingly. If a user has requested their account to be deleted, we will set their
/// username to be `deleted_{player.id}` where player.id will be the UUID of the player without
/// slashes._
///
/// _**Note:** Once a player account has been merged or deleted, it cannot be restored with the same ID._
#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct Player {
    /// [Limit: 36 Char] A player's ID
    pub id: Uuid,
    /// [Limit: 64 Char] A player's local username
    pub name: String,
    /// A player's home team (the original team they played for)
    ///
    /// _**Note:** This team may no longer be active, as it may have been eliminated_
    // Developer note: This field is not deserialized at load, it must be populated after the fact
    // from `main_team`
    #[serde(skip_deserializing)]
    pub team: Option<SimpleTeam>,
    /// A player's guest team (the team they're currently playing for)
    ///
    /// _**Note:** A null `active_team` but non-null `team` means the player's team has been
    /// eliminated and they have yet to choose a new team._
    // Developer note: This field is not deserialized at load, it must be populated after the fact
    // from `playing_for`
    #[serde(skip_deserializing)]
    pub active_team: Option<SimpleTeam>,
    /// The star ratings [1<=x<=5] for a player
    pub ratings: Rating,
    /// The statistics for a player
    pub stats: Stat,
    /// Whether a player has been flagged globally as an alt
    pub is_alt: bool,
    /// When a player was created
    ///
    /// _**Note:** If a player was created prior to January 1, 2023 (or the start of Season 3), then their
    /// creation date reflects the day their account was migrated, not the day they first signed
    /// up._
    pub created: NaiveDateTime,
    /// When a player was last updated
    ///
    /// _**Note:** This is enforced by the database and will likely be updated nightly (indicating
    /// that the user made a move and therefore had their statistics updated)._
    pub updated: NaiveDateTime,
    /// The player who created this player. This should either be the System user (UUID: `a147b32b-6779-462c-b20b-5f5bef4702fa`) or the MigrationUser (UUID: `be48ffec-e101-4d7c-9880-c2b25e86c355`).
    pub createdby: Uuid,
    /// The player who last updated this player. This could be the System user (UUID: `a147b32b-6779-462c-b20b-5f5bef4702fa`), the MigrationUser (UUID: `be48ffec-e101-4d7c-9880-c2b25e86c355`), an Admin, or potentially the user themself.
    pub updatedby: Uuid,
    #[serde(skip_serializing)]
    pub must_captcha: bool,
    #[serde(skip_serializing)]
    pub main_team: Option<i32>,
    #[serde(skip_serializing)]
    pub playing_for: Option<i32>,
}

/// # Move
/// Gather metadata and related objects for a player
#[derive(Queryable, Serialize, Deserialize, Debug, JsonSchema)]
#[diesel(table_name = schema::move_)]
pub struct Move {
    id: Uuid,
    territory_id: i32,
    is_mvp: i32,
    power: f64,
    multiplier: f64,
    weight: f64,
    stars: i32,
    team_id: i32,
    is_merc: bool,
    turn_id: i32,
    created: NaiveDateTime,
    updated: NaiveDateTime,
}

impl Move {
    fn available_move_to_player(player: Player, turn_id: Option<i32>) {}
}

/// # Player With Related Objects
/// Gather metadata and related objects for a player
#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct Player360 {
    /// Metadata for a player
    player: Player,
    /// A list of moves made by the player over all season
    moves: Vec<Move>,
    /// A list of awards bequeathed to the player
    awards: Vec<AwardInfo>,
    /// A list of the player's publicly-released platform connections
    links: Vec<Link>,
}

/// # Team Colors
#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct Color {
    /// The rgba primary (foreground) color of the team
    /// Format: `rgba(red, green, blue, alpha)`
    /// where red, green, blue, and alpha 0.0<=x<=255.0
    primary_color: String,
    /// The rgba primary (accent) color of the team
    /// Format: `rgba(red, green, blue, alpha)`
    /// where red, green, blue, and alpha 0.0<=x<=255.0
    secondary_color: String,
}

/// # Team Metadata
/// Full metadata of a team
#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct Team {
    id: i32,
    name: String,
    colors: Color,
    logo: String,
    seasons: Vec<i32>,
    created: NaiveDateTime,
    updated: NaiveDateTime,
}

// TODO: Add star rating mappings here
/// # Player Star Rating
/// A set of ratings (1-5) for a player
#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct Rating {
    /// [Limit: 1<=x<=5] The overall star rating of the player, which is the up-rounded median of the other ratings
    overall: i32,
    /// [Limit: 1<=x<=5] The star rating for the number of turns made by the player in all games
    turns: i32,
    /// [Limit: 1<=x<=5] The star rating for the number of turns made by the player this game
    game_turns: i32,
    /// [Limit: 1<=x<=5] The star rating for the number of times the player has been mvp in all
    /// games
    mvps: i32,
    /// [Limit: 1<=x<=5] The star rating for the number of consecutive turns the player has made
    streak: i32,
}

/// # Player Statistic
/// A set of statistics about a player
#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct Stat {
    /// The total number of turns made by the player in all games
    turns: i32,
    /// The total number of turns made by the player in this game
    game_turns: i32,
    /// The total number of times the player has been mvp in all games
    mvps: i32,
    /// The total number of consecutive turns the player has made
    streak: i32,
}

/// # Player Linked Accounts
/// The usernames on connected platforms the user has made publicly available
#[derive(Queryable, Serialize, Deserialize, Debug, JsonSchema)]
#[diesel(belongs_to(User))]
pub struct Link {
    /// [Limit: 10 Char] The name of the platform
    platform: String,
    /// [Limit: 256 Char] The name of the user
    username: String,
}

/// # Award
/// Information pertaining to an award given to a user
#[derive(Queryable, Serialize, Deserialize, Debug, JsonSchema)]
#[diesel(belongs_to(User))]
pub struct AwardInfo {
    id: i32,
    name: String,
    info: String,
}

/// # List of all players, including id, team, and name for all time
/// Returns all players, but provides simplified data structure for smaller payload size. Unlike other methods, this one will return before a player has been part of a roll.
#[openapi(tag = "Player", ignore = "conn")]
#[get("/players")]
pub(crate) async fn players(conn: DbConn) -> Result<Json<Vec<SimplePlayer>>> {
    todo!()
}

/// # List of all active players, including id, team, and name for all teams
/// Returns all active players, but provides simplified data structure for smaller payload size. Unlike other methods, this one will return before a player has been part of a roll.
#[openapi(tag = "Player", ignore = "conn")]
#[get("/players/active")]
pub(crate) async fn players_active(conn: DbConn) -> Result<Json<Vec<SimplePlayer>>> {
    todo!()
}

/// # Search for player(s) by partial name
/// Search for player(s) by partial name
#[openapi(tag = "Player", ignore = "conn")]
#[get("/players/search/<query>?<limit>")]
pub(crate) async fn player_search(
    mut query: String,
    limit: Option<i32>,
    conn: DbConn,
) -> Result<Json<Vec<SimplePlayer>>> {
    todo!()
}

/// # Retrieve metadata for a player
/// **Note:** this requires a `player`'s `id` to be sent, which can be obtained from any query returning a
/// `SimplePlayer` such as `/players/search/<query>` or
/// `/players` or `/team/{team.id}/players` or `/team/{team.id}/mercs`
#[openapi(tag = "Player", ignore = "conn")]
#[get("/player/<player_id>")]
pub(crate) async fn player_meta(player_id: Uuid, conn: DbConn) -> Result<Json<Player>> {
    todo!()
}

/// # Retrieve available moves for a player for a given turn
///
/// _**Note:** this requires a `player`'s `id` to be sent, which can be obtained from any query returning a
/// `SimplePlayer` such as `/players/search/<query>` or
/// `/players` or `/team/{team.id}/players` or `/team/{team.id}/mercs`_
#[openapi(tag = "Player", ignore = "conn")]
#[get("/player/<player_id>/available_moves?<turn_id>")]
pub(crate) async fn available_player_moves(
    player_id: Uuid,
    turn_id: Option<i32>,
    conn: DbConn,
) -> Result<Json<Vec<Territory>>> {
    todo!()
}

/// # Retrieve moves for a player
///
/// _**Note:** this requires a `player`'s `id` to be sent, which can be obtained from any query returning a
/// `SimplePlayer` such as `/players/search/<query>` or
/// `/players` or `/team/{team.id}/players` or `/team/{team.id}/mercs`_
#[openapi(tag = "Player", ignore = "conn")]
#[get("/player/<player_id>/moves")]
pub(crate) async fn player_moves(player_id: Uuid, conn: DbConn) -> Result<Json<Vec<Move>>> {
    todo!()
}

/// # Retrieve awards for a player
///
/// _**Note:** this requires a `player`'s `id` to be sent, which can be obtained from any query returning a
/// `SimplePlayer` such as `/players/search/<query>` or
/// `/players` or `/team/{team.id}/players` or `/team/{team.id}/mercs`_
#[openapi(tag = "Player", ignore = "conn")]
#[get("/player/<player_id>/awards")]
pub(crate) async fn player_awards(player_id: Uuid, conn: DbConn) -> Result<Json<Vec<AwardInfo>>> {
    todo!()
}

/// # Retrieve roles for a player
///
/// _**Note:** this requires a `player`'s `id` to be sent, which can be obtained from any query returning a
/// `SimplePlayer` such as `/players/search/<query>` or
/// `/players` or `/team/{team.id}/players` or `/team/{team.id}/mercs`_
#[openapi(tag = "Player", ignore = "conn")]
#[get("/player/<player_id>/roles")]
pub(crate) async fn player_roles(player_id: Uuid, conn: DbConn) -> Result<Json<Vec<Role>>> {
    todo!()
}

/// # Retrieve publicly linked accounts for a player
///
/// _**Note:** this requires a `player`'s `id` to be sent, which can be obtained from any query returning a
/// `SimplePlayer` such as `/players/search/<query>` or
/// `/players` or `/team/{team.id}/players` or `/team/{team.id}/mercs`_
#[openapi(tag = "Player", ignore = "conn")]
#[get("/player/<player_id>/links")]
pub(crate) async fn player_links(player_id: Uuid, conn: DbConn) -> Result<Json<Vec<Link>>> {
    todo!()
}

/// # Retrieve player info, moves, awards for 1-100 players at once
///
/// Batch retrieval of `players` - `players` should be a comma-separated list of up to and including 100 player.id without spaces.
#[openapi(tag = "Player", ignore = "conn")]
#[get("/players/batch?<players>")]
pub(crate) async fn player_batch(players: Uuid, conn: DbConn) -> Result<Json<Vec<Player360>>> {
    todo!()
}

// Todo: Region

#[derive(FromForm, Serialize, Deserialize, Debug)]
pub struct PrimitiveDateTime(time::PrimitiveDateTime);

impl JsonSchema for PrimitiveDateTime {
    fn schema_name() -> String {
        "Date Time".to_string()
    }
    fn json_schema(gen: &mut SchemaGenerator) -> schemars::schema::Schema {
        SchemaObject {
            instance_type: Some(InstanceType::String.into()),
            array: None,
            ..Default::default()
        }
        .into()
    }
}

// Event log
/// # Retrieve a list of events
///
/// Parameters:
///
/// - **since (DateTime String; "2023-01-01T13:41:00")**: Get data after this UTC timestamp
///
/// - **after (Uuid)**: Get data created after this Event id
///
/// - **event_type (Array of EventType; `["PlayerCreate", "PlayerMerge"]`)**: Only get events matching these
/// event types
///
/// - **count (i32; 1<=x<=250)**: How many Events to retrieve (defaults to 100, can be no more than 250)
///
/// - **page (i32)**: Which page to grab (used with Count)
///
#[openapi(tag = "Event", ignore = "conn")]
#[get("/events?<since>&<after>&<event_type>&<count>&<page>")]
pub(crate) async fn events(
    since: Option<PrimitiveDateTime>,
    after: Option<Uuid>,
    event_type: Option<Vec<EventType>>,
    count: i32,
    page: i32,
    conn: DbConn,
) -> Result<Json<Vec<Player360>>> {
    todo!()
}

/// # Turn
/// Metadata for a specific turn
#[derive(Selectable, Queryable, Serialize, Deserialize, Debug, JsonSchema)]
#[diesel(table_name = schema::turn)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Turn {
    id: i32,
    season: i32,
    day: i32,
    complete: bool,
    active: bool,
    finale: bool,
    rerolls: i32,
    roll_start: NaiveDateTime,
    roll_end: Option<NaiveDateTime>,
    all_or_nothing: bool,
    map: Option<String>,
    random_seed: Option<f64>,
    created: NaiveDateTime,
    updated: NaiveDateTime,
    createdby: Uuid,
    updatedby: Uuid,
}

impl Turn {
    /// Returns a list of turns (aka rolls); optionally filtered by season with `season_filter`
    pub fn all(
        season_filter: Option<i32>,
        conn: &mut PgConnection,
    ) -> crate::error::Result<Vec<Self>> {
        use crate::schema::turn::{dsl::turn, id, season};
        let mut query = turn
            .select(Self::as_select())
            .order_by(id.asc())
            .into_boxed();

        // If a season was specified, then return only the items in that season
        if let Some(season_filter) = season_filter {
            query = query.filter(season.eq(season_filter));
        }

        query.load(conn).map_rre()
    }
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct Turn360 {
    turn: Turn,
    events: Event,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema, FromFormField)]
pub enum EventType {
    /// A player has been updated
    PlayerCreate,
    /// A player has been updated
    PlayerNameUpdate,
    /// A player has joined a team
    ///
    /// _**Note:** This only tracks `playing_for` team changes_
    PlayerTeamUpdate,
    /// A player has received an Award
    PlayerAward,
    /// A player has been merged
    ///
    /// - `before` indicates user's old id
    /// - `after` indicates user's new id
    PlayerMerge,
    /// A player has been deleted
    PlayerDelete,
    /// A territory has been decided for a turn
    ///
    /// - `before` indicates territory's old owner
    ///
    /// - `after` indicates territory's new owner
    ///
    /// - `description` is a string containing:
    ///
    ///     - `value`: the value used to determine the territory ownership
    ///
    TerritoryDecision,
    /// A reroll has occured for a territory
    ///
    /// - `before` indicates territory's old owner
    ///
    /// - `after` indicates territory's new owner
    ///
    /// - `description` is a string containing:
    ///
    ///     - `value`: the value used to determine the territory ownership
    ///     
    /// _**Note:** A reroll will revoke all user moves made on the territory for the upcoming turn. A
    /// `Notification` will also be published to impacted users._
    ///
    TerritoryReroll,
}

/// # System Event
///
/// Represents an event which users may wish to be notified about/track.
///
/// _**Note:** Each `EventType` (`event_type`) has its own meaning for `before`, `after`, and
/// `description`. See the documentation for `EventType` to know what those fields mean within the
/// context of that `EventType`._
#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct Event {
    id: Uuid,
    event_type: EventType,
    before: Option<String>,
    after: Option<String>,
    description: Option<String>,
    active_turn: Option<i32>,
    created: NaiveDateTime,
    createdby: Uuid,
    updated: NaiveDateTime,
    updatedby: Uuid,
}

/// # List of all rolls, either for all seasons or just one.
///
/// Returns information about all turns, or just the turns specified in `season if it is provided
#[openapi(tag = "Turn", ignore = "conn")]
#[get("/turns?<season>")]
pub(crate) async fn turns(season: Option<i32>, conn: DbConn) -> Result<Json<Vec<Turn>>> {
    conn.run(move |c| Turn::all(season, c)).await.map(Json)
}

/// # Retrieve audit log for a turn/roll.
#[openapi(tag = "Turn", ignore = "conn")]
#[get("/turn/<turn_id>")]
pub(crate) async fn turn_log(turn_id: i32, conn: DbConn) -> Result<Json<Vec<Turn360>>> {
    todo!()
}

/// # List of all teams
///
/// Returns a list of all teams, including those from past seasons
#[openapi(tag = "Team", ignore = "conn")]
#[get("/teams")]
pub(crate) async fn teams(conn: DbConn) -> Result<Json<Vec<Team>>> {
    todo!()
}

#[openapi(tag = "Team", ignore = "conn")]
#[get("/team/<team_id>/stats/history")]
pub(crate) async fn team_stat_history(team_id: i32, conn: DbConn) -> Result<Json<Vec<Team>>> {
    todo!()
}

#[openapi(tag = "Team", ignore = "conn")]
#[get("/team/<team_id>/stats")]
pub(crate) async fn team_stats(team_id: i32, conn: DbConn) -> Result<Json<Vec<Turn360>>> {
    todo!()
}

#[openapi(tag = "Team", ignore = "conn")]
#[get("/teams/leaderboard?<turn_id>")]
pub(crate) async fn team_leaderboard(
    turn_id: Option<i32>,
    conn: DbConn,
) -> Result<Json<Vec<Turn360>>> {
    todo!()
}

#[openapi(tag = "Team", ignore = "conn")]
#[get("/teams/search/<query>")]
pub(crate) async fn team_search(
    mut query: String,
    // TODO: Limit the length of this
    conn: DbConn,
) -> Result<Json<Vec<SimpleTeam>>> {
    todo!()
}

#[openapi(tag = "Team", ignore = "conn")]
#[get("/team/<team_id>/players")]
pub(crate) async fn team_players(team_id: i32, conn: DbConn) -> Result<Json<Vec<Player>>> {
    todo!()
}

#[openapi(tag = "Team", ignore = "conn")]
#[get("/team/<team_id>/mercs")]
pub(crate) async fn team_mercs(team_id: i32, conn: DbConn) -> Result<Json<Vec<Player>>> {
    todo!()
}

/// # Territory
#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct Territory {
    id: Uuid,
    name: String,
    region: Region,
}

/// # Region
#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct Region {
    id: Uuid,
    name: String,
}

/// # Team Odds for Winning Territory
#[derive(Serialize, Deserialize, Debug, JsonSchema)]
pub struct TeamOdd {
    territory: Territory,
    occupier: SimpleTeam,
    winner: SimpleTeam,
    mvp: SimplePlayer,
    players: i32,
    ones: i32,
    twos: i32,
    threes: i32,
    fours: i32,
    fives: i32,
    team_power: i32,
    territory_power: i32,
    team_chance: i32,
}

#[openapi(tag = "Team", ignore = "conn")]
#[get("/team/<team_id>/odds/<turn_id>")]
pub(crate) async fn team_odds(
    team_id: i32,
    turn_id: i32,
    conn: DbConn,
) -> Result<Json<Vec<TeamOdd>>> {
    todo!()
}

#[openapi(tag = "Team", ignore = "conn")]
#[get("/team/<team_id>/territories_visited/<season>")]
pub(crate) async fn territories_visited(
    team_id: i32,
    season: i32,
    conn: DbConn,
) -> Result<Json<Vec<Territory>>> {
    todo!()
}

//Territory
// /territories
#[openapi(tag = "Territory", ignore = "conn")]
#[get("/territories")]
pub(crate) async fn territories(conn: DbConn) -> Result<Json<Vec<Territory>>> {
    todo!()
}
// /territories/search/<query>
#[openapi(tag = "Territory", ignore = "conn")]
#[get("/territories/search/<query>")]
pub(crate) async fn territory_search(
    mut query: String,
    // TODO: Limit the length of this
    conn: DbConn,
) -> Result<Json<Vec<Territory>>> {
    todo!()
}
// /territories/heat/<turn_id>
// /territories/ownership/<turn_id>
// /territory/<territory_id>/ownership
// /territory/<territory_id>/ownership/<turn_id>
// /territory/<territory_id>/moves/<turn_id>
#[openapi(tag = "Territory", ignore = "conn")]
#[get("/territory/<territory_id>/moves/<turn_id>")]
pub(crate) async fn territory_moves_by_turn(
    territory_id: i32,
    turn_id: i32,
    // TODO: Limit the length of this
    conn: DbConn,
) -> Result<Json<Vec<Move>>> {
    todo!()
}
// /territory/<territory_id>
// /territory/<territory_id>/neighbors
#[openapi(tag = "Territory", ignore = "conn")]
#[get("/territory/<territory_id>/neighbors")]
pub(crate) async fn territory_neighbors(
    mut territory_id: i32,
    // TODO: Limit the length of this
    conn: DbConn,
) -> Result<Json<Vec<Territory>>> {
    todo!()
}

// Case
#[derive(Serialize, Deserialize, Debug, JsonSchema, FromFormField)]
pub enum CaseStatus {
    /// Case has been created, but has not yet been acted upon
    Open,
    /// Case is awaiting on user action
    WaitingOnUser,
    /// Case is in progress
    InProgress,
    /// Case has been completed
    ClosedCompletd,
    /// Case has been rejected
    ClosedRejected,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema, FromFormField)]
pub enum CaseType {
    /// Update team (home)
    AccountUpdateTeam,
    /// Update team (guest)
    AccountUpdatePlayingFor,
    /// Update teams (both home and guest)
    AccountUpdateTeams,
    /// Delete account
    AccountDelete,
    /// Merge account (when there are overlapping objects)
    AccountMerge,
    /// Report an account (see Description for why)
    AccountReport,
    /// Request streak reinstatement
    AccountStreak,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema, FromForm)]
pub struct Case {
    id: Uuid,
    status: CaseStatus,
    case_type: CaseType,
    description: String,
    created: PrimitiveDateTime,
    updated: PrimitiveDateTime,
    createdby: Uuid,
    updatedby: Uuid,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema, FromForm)]
pub struct Notification {
    id: Uuid,
    title: String,
    icon: String,
    body: String,
    sender: Uuid,
    linked_case: Option<Uuid>,
    permit_text_response: bool,
    require_response: bool,
    created: PrimitiveDateTime,
    updated: PrimitiveDateTime,
    createdby: Uuid,
    updatedby: Uuid,
}

#[derive(Queryable, Serialize, Deserialize, Debug, JsonSchema, FromForm)]
#[diesel(belongs_to(User))]
#[diesel(belongs_to(Role))]
#[diesel(belongs_to(Notification))]
pub struct NotificationRecipient {
    id: Uuid,
    notification: Uuid,
    user_id: Option<Uuid>,
    role_id: Option<Uuid>,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema, FromFormField)]
pub enum NotificationResponse {
    Acknowledged,
    Accepted,
    Declined,
    Dismissed,
}

#[derive(Queryable, Serialize, Deserialize, Debug, JsonSchema, FromForm)]
#[diesel(belongs_to(NotificationRecipient))]
pub struct NotificationReceipt {
    id: Uuid,
    notification_recipient: Uuid,
    user_id: Uuid,
    response: Option<NotificationResponse>,
}

#[openapi(tag = "Case", ignore = "conn")]
#[get("/cases")]
pub(crate) async fn get_cases(conn: DbConn) -> Result<Json<Vec<Case>>> {
    todo!()
}

#[openapi(tag = "Case", ignore = "conn")]
#[post("/case", data = "<case>")]
pub(crate) async fn create_case(case: Form<Case>, conn: DbConn) -> Result<Json<Case>> {
    todo!()
}

// Todo: add limits on how many cases can be created per day and per user
#[openapi(tag = "Case", ignore = "conn")]
#[patch("/case/<case_id>", data = "<case>")]
pub(crate) async fn update_case(
    case: Form<Case>,
    case_id: Uuid,
    conn: DbConn,
) -> Result<Json<Vec<Territory>>> {
    todo!()
}

#[openapi(tag = "Case", ignore = "conn")]
#[get("/case/<case_id>/notifications")]
pub(crate) async fn get_case_notifications(case_id: Uuid, conn: DbConn) -> Result<Json<Vec<Case>>> {
    todo!()
}

// Todo: add limits on how many notifications can be created per day, per user, and per case
#[openapi(tag = "Case", ignore = "conn")]
#[post("/case/<case_id>/notification", data = "<notification>")]
pub(crate) async fn create_case_notification(
    case_id: Uuid,
    notification: Form<Notification>,
    conn: DbConn,
) -> Result<Json<Vec<Case>>> {
    todo!()
}

// Notification
// Todo: restrict who can create what types of notifications and two whom
#[openapi(tag = "Notification", ignore = "conn")]
#[post("/notification", data = "<notification>")]
pub(crate) async fn create_notification(
    notification: Form<Notification>,
    conn: DbConn,
) -> Result<Json<Notification>> {
    todo!()
}

#[openapi(tag = "Notification", ignore = "conn")]
#[get("/notifications")]
pub(crate) async fn get_notifications(conn: DbConn) -> Result<Json<Vec<Notification>>> {
    todo!()
}
