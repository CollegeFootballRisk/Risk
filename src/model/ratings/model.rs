/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use crate::model::stats::Stats;
use schemars::JsonSchema;
/// The star ratings (1-5) for a Player
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct Ratings {
    /// The rounded-up median of the other ratings
    pub overall: i32,
    /// The rating/stars (1-5) for number of turns submitted in all seasons by a Player
    pub totalTurns: i32,
    /// The rating/stars (1-5) for number of turns submitted this seasons by a Player
    pub gameTurns: i32,
    /// The rating/stars (1-5) for number of turns submitted in all seasons by a Player for which they were the MVP
    pub mvps: i32,
    /// The rating/stars (1-5) for number of consecutive turns submitted by a Player
    pub streak: i32,
}

impl Ratings {
    pub(crate) fn load(stat: &Stats) -> Ratings {
        let totalTurns = Self::fromarr(stat.totalTurns, [0, 10, 25, 50, 100]);
        let gameTurns = Self::fromarr(stat.gameTurns, [0, 5, 10, 25, 40]);
        let mvps = Self::fromarr(stat.mvps, [0, 1, 5, 10, 25]);
        let streak = Self::fromarr(stat.streak, [0, 3, 5, 10, 25]);
        let mut numbers = vec![totalTurns, gameTurns, mvps, streak]; // awards
        numbers.sort_unstable();
        let mid = ((numbers[1] as f32 + numbers[2] as f32) / 2_f32).round() as i32;
        let overall: i32 = mid;
        Ratings {
            overall,
            totalTurns,
            gameTurns,
            mvps,
            streak,
        }
    }

    fn fromarr(num: i32, arr: [i32; 5]) -> i32 {
        let mut r = 0;
        for x in &arr {
            if x <= &num {
                r += 1;
            } else {
                r += 0;
                break;
            }
        }
        r
    }
}
