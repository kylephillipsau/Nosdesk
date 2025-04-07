use diesel::prelude::*;
use diesel::QueryResult;

use crate::db::DbConnection;
use crate::models::*;
use crate::schema::*;

// Device operations
pub fn get_all_devices(conn: &mut DbConnection) -> QueryResult<Vec<Device>> {
    devices::table
        .order_by(devices::id.asc())
        .load::<Device>(conn)
}

pub fn get_device_by_id(conn: &mut DbConnection, device_id: i32) -> QueryResult<Device> {
    devices::table
        .find(device_id)
        .first(conn)
}

pub fn get_device_by_ticket_id(conn: &mut DbConnection, ticket_id: i32) -> QueryResult<Device> {
    devices::table
        .filter(devices::ticket_id.eq(ticket_id))
        .first(conn)
}

pub fn create_device(conn: &mut DbConnection, new_device: NewDevice) -> QueryResult<Device> {
    diesel::insert_into(devices::table)
        .values(&new_device)
        .get_result(conn)
}

pub fn update_device(conn: &mut DbConnection, device_id: i32, device: NewDevice) -> QueryResult<Device> {
    diesel::update(devices::table.find(device_id))
        .set(&device)
        .get_result(conn)
}

pub fn delete_device(conn: &mut DbConnection, device_id: i32) -> QueryResult<usize> {
    diesel::delete(devices::table.find(device_id))
        .execute(conn)
} 