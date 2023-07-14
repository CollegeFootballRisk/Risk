use uuid::Uuid;
use chrono::NaiveDateTime;
use diesel::prelude::*;


#[derive(default,debug,queryable,identifiable,insertable,serialize,deserialize)]
#[diesel(primary_key(id))]
#[diesel(table_name = role)]
pub struct Role {
    #[diesel(deserialize_as = "i32")]
    pub id: Option<i32>,
    pub name: String,
    pub created: NaiveDateTime,
    pub updated: NaiveDateTime,
    pub createdby: Uuid,
    pub updatedby: Uuid,
}


#[derive(default,debug,queryable,identifiable,insertable,serialize,deserialize)]
#[diesel(belongs_to(Permission))]
#[diesel(belongs_to(Role))]
#[diesel(table_name = role_permission)]
#[diesel(primary_key(permission_id, role_id))]
pub struct RolePermission {
    
}

#[derive(default,debug,queryable,identifiable,insertable,serialize,deserialize)]
#[diesel(primary_key(id))]
#[diesel(table_name = permission)]
pub struct Permission {
    #[diesel(deserialize_as = "i32")]
    pub id: Option<i32>,
    pub name: String,
    pub created: NaiveDateTime,
    pub updated: NaiveDateTime,
    pub createdby: Uuid,
    pub updatedby: Uuid,
}

impl Role {
    pub fn get_permissions(&self, &mut PgConnection) -> Vec<Permission> {
        
    }
}

impl Permission {
    pub fn get_roles(&self, &mut PgConnection) -> Vec<Role> {

    }
}
