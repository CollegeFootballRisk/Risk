/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use crate::schema::regions;
//use diesel::pg::expression::dsl::array;
use diesel::prelude::*;
use diesel_citext::types::CiString;
use schemars::JsonSchema;
use std::result::Result;

#[derive(Serialize, Queryable, Deserialize, JsonSchema)]
pub(crate) struct Region {
    id: i32,
    name: CiString,
    /*#[serde(skip_serializing)]
    territories: Vec<String>*/
}

impl Region {
    pub(crate) fn load(conn: &PgConnection) -> Result<Vec<Region>, diesel::result::Error> {
        regions::table
            //.inner_join(territories::table.on(territories::region.eq(regions::id)))
            //.group_by(regions::id)
            //.group_by(regions::name)
            .select((
                regions::id,
                regions::name,
                //array(territories::name)
            ))
            .load::<Region>(conn)

        // Unfortunately, the currently-used diesel doesn't support Arrays with groupby, and rocket_dbs / mautamu/diesel-citext don't support the latest diesel
    }
}
