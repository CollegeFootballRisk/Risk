# Deviations from CollegeFootballRisk's API
  - /*
    > All endpoints are presently CaSe SeNsiTiVe. We're working on a patch for this, as the expected behaviour is to ignore case.

  - /players/batch?players=comma,separated,list
    >To retrieve multiple players at once, please use this request. Players can be batched in unlimited number (Rocket places a character limit at 32 KiB), but we ask that you keep it to around 100 players. This will return the same as /player?player=String, except will be bound in an array. Note: the _players_ parameter should not have spaces before or after the comma unless that username includes a leading or trailing space. API returns in order of user_id, not order placed into list!
    > The CFB api does not include 'platform.' Platform indicates a player's platform of entry (which platform their 'name' originates from). The current options are (string): _internal, reddit_. Future options may include: _twitter, groupme, discord_.

  - /player->stats
    >The model in the CFB api states that totalTurns, gameTurns, mvps, and streak should be integers, but the api returns strings. We decided to follow what the model says should happen instead of the actual behaviour. Since most programmes are written in Python/Excel/etc which are liable to not care about the actual data type present, we do not expect this to be an issue.

  - /player->platform
    >We intend to test a few things, such as the possibility of using Discord OAUTH for authentication. We have therefore included a platform tag. Platforms will _not_ be included in the player->name or team->player->player strings, but we will probably present them on the final GUI as "reddit/u/mautamu" vs "@mautamu#Discord" to differentiate between Reddit, Discord, etc . . .

  - /players
    >The model in the CFB api states that turnsPlayed and mvps should both appear on the /players endpoint. Similarly, both should be integers. We decided to follow what the model says should happen instead of the actual behaviour. Since most programmes are written in Python/Excel/etc which are liable to not care about the actual data type present, we do not expect this to be an issue.
    > Another issue one may encounter, especially with team names, is that GET variables MUST be encoded. ?team=Texas A&M will not return valid results. To achieve the same result as CollegeFootballRisk, you will need to use ?team=Texas%20A%26M. Spaces are allowed, so ?team=Texas A%26M will work.


  - /turns
    >The CFB api does not include `finale.` Finale is for use with determining whether that day was the last day of that season/game.

  - /stats/
    > No differences at present.

  - /team/players
    > The only difference is the presence of the 'id' tag. It is not important and can be disregarded.

  - /*
    > We use rgba values rather than hex values