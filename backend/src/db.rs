use diesel::pg::PgConnection;
use diesel::r2d2::{self, ConnectionManager};
use dotenv::dotenv;
use std::env;

pub type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;
pub type DbConnection = r2d2::PooledConnection<ConnectionManager<PgConnection>>;

pub fn establish_connection_pool() -> Pool {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    
    // Configure the connection pool with reasonable limits
    r2d2::Pool::builder()
        .max_size(10)  // Limit to 10 connections
        .min_idle(Some(1))  // Maintain at least 1 idle connection
        .build(manager)
        .unwrap_or_else(|e| {
            eprintln!("Error creating connection pool: {}", e);
            panic!("Database connection error: {}", e);
        })
} 