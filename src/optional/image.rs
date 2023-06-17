/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::structs::TerritoryOwnersInsert;
use diesel::pg::PgConnection;
use std::collections::HashMap;

/// Generate an image, if necessary.
#[cfg(feature = "risk_image")]
pub fn make_image(territories: &[TerritoryOwnersInsert], conn: &mut PgConnection) {
    use crate::structs::Team;
    extern crate image;
    extern crate nsvg;
    // first we got get the SVG image
    use std::fs;
    let teams = Team::load(conn);
    let mut vec = fs::read_to_string("resources/map.svg").unwrap();
    let base: String = "{{?}}".to_owned();
    let mut team_map = HashMap::new();
    match teams {
        Ok(teams) => {
            for team in teams {
                team_map.insert(team.id, team.color);
            }
            for item in territories {
                vec = vec.replace(
                    &base.replace('?', &item.territory_id.to_string()),
                    team_map.get(&item.owner_id).unwrap(),
                );
            }
            let svg = nsvg::parse_str(&vec, nsvg::Units::Pixel, 96.0).unwrap();
            let image = svg.rasterize(2.0).unwrap();
            let (width, height) = image.dimensions();
            image::save_buffer(
                "../server/static/images/curr_map.png",
                &image.into_raw(),
                width,
                height,
                image::ColorType::Rgba8,
            )
            .expect("Failed to save png.");
        }
        Err(e) => {
            dbg!(e);
        }
    }
}
