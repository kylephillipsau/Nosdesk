use diesel::prelude::*;
use diesel::result::Error;

use crate::db::DbConnection;
use crate::models::*;
use crate::schema::*;

// User repository functions
pub fn get_users(conn: &mut DbConnection) -> Result<Vec<User>, Error> {
    users::table
        .order_by(users::name.asc())
        .load::<User>(conn)
}

pub fn get_user_by_id(id: i32, conn: &mut DbConnection) -> Result<User, Error> {
    users::table
        .find(id)
        .first::<User>(conn)
}

pub fn get_user_by_uuid(uuid: &str, conn: &mut DbConnection) -> Result<User, Error> {
    users::table
        .filter(users::uuid.eq(uuid))
        .first::<User>(conn)
}

pub fn get_user_by_email(email: &str, conn: &mut DbConnection) -> Result<User, Error> {
    users::table
        .filter(users::email.eq(email))
        .first::<User>(conn)
}

pub fn create_user(
    user: NewUser,
    conn: &mut DbConnection,
) -> Result<User, Error> {
    diesel::insert_into(users::table)
        .values(user)
        .get_result(conn)
}

pub fn update_user(
    id: i32,
    user: UserUpdate,
    conn: &mut DbConnection,
) -> Result<User, Error> {
    diesel::update(users::table.find(id))
        .set(user)
        .get_result(conn)
}

pub fn delete_user(id: i32, conn: &mut DbConnection) -> Result<usize, Error> {
    diesel::delete(users::table.find(id)).execute(conn)
} 