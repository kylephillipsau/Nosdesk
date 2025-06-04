use diesel::pg::PgConnection;
use diesel::r2d2::{self, ConnectionManager};
use dotenv::dotenv;
use std::env;
use std::time::Duration;

pub type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;
pub type DbConnection = r2d2::PooledConnection<ConnectionManager<PgConnection>>;

pub fn establish_connection_pool() -> Pool {
    dotenv().ok();
    
    // Validate that DATABASE_URL is set and not empty
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL environment variable must be set");
    
    if database_url.trim().is_empty() {
        panic!("DATABASE_URL cannot be empty");
    }

    // Security: Warn if using insecure connection in production
    if !database_url.contains("sslmode=") {
        eprintln!("⚠️  WARNING: DATABASE_URL does not specify SSL mode. Consider adding sslmode=require for production.");
    }

    // Get database pool configuration from environment
    let max_connections = env::var("DB_MAX_CONNECTIONS")
        .unwrap_or("10".to_string())
        .parse::<u32>()
        .unwrap_or(10)
        .clamp(1, 50); // Reasonable limits

    let min_connections = env::var("DB_MIN_CONNECTIONS")
        .unwrap_or("1".to_string())
        .parse::<u32>()
        .unwrap_or(1)
        .clamp(0, max_connections);

    let connection_timeout = env::var("DB_CONNECTION_TIMEOUT")
        .unwrap_or("30".to_string())
        .parse::<u64>()
        .unwrap_or(30)
        .clamp(5, 300); // 5 seconds to 5 minutes

    println!("Database Configuration:");
    println!("  Max connections: {}", max_connections);
    println!("  Min connections: {}", min_connections);
    println!("  Connection timeout: {}s", connection_timeout);

    let manager = ConnectionManager::<PgConnection>::new(database_url);
    
    // Configure the connection pool with security-conscious settings
    r2d2::Pool::builder()
        .max_size(max_connections)
        .min_idle(if min_connections > 0 { Some(min_connections) } else { None })
        .connection_timeout(Duration::from_secs(connection_timeout))
        .idle_timeout(Some(Duration::from_secs(300))) // Close idle connections after 5 minutes
        .max_lifetime(Some(Duration::from_secs(1800))) // Recreate connections every 30 minutes
        .build(manager)
        .unwrap_or_else(|e| {
            eprintln!("Error creating database connection pool: {}", e);
            panic!("Database connection error: {}", e);
        })
} 