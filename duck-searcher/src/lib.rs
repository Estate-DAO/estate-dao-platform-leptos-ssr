use std::path::PathBuf;
use std::sync::OnceLock;

use duckdb::{
    arrow::{record_batch::RecordBatch, util::pretty::print_batches},
    r2d2::{Pool, PooledConnection},
    DuckdbConnectionManager, Result,
};

type ConnectionPool = Pool<DuckdbConnectionManager>;
type PooledConn = PooledConnection<DuckdbConnectionManager>;

static POOL: OnceLock<ConnectionPool> = OnceLock::new();

pub fn get_parquet_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../city.parquet")
}

fn create_connection_pool() -> Result<ConnectionPool> {
    let manager = DuckdbConnectionManager::memory()?;
    let pool = Pool::builder().max_size(10).build(manager).map_err(|e| {
        duckdb::Error::SqliteFailure(
            duckdb::ffi::Error::new(duckdb::ffi::SQLITE_MISUSE),
            Some(e.to_string()),
        )
    })?;

    // <!-- Initialize parquet extension on a connection from pool -->
    let conn = pool.get().map_err(|e| {
        duckdb::Error::SqliteFailure(
            duckdb::ffi::Error::new(duckdb::ffi::SQLITE_MISUSE),
            Some(e.to_string()),
        )
    })?;
    conn.execute_batch("INSTALL parquet; LOAD parquet;")?;
    drop(conn);

    Ok(pool)
}

fn get_connection() -> Result<PooledConn> {
    let pool =
        POOL.get_or_init(|| create_connection_pool().expect("Failed to create connection pool"));

    pool.get().map_err(|e| {
        duckdb::Error::SqliteFailure(
            duckdb::ffi::Error::new(duckdb::ffi::SQLITE_MISUSE),
            Some(e.to_string()),
        )
    })
}

pub fn search_cities_by_prefix(prefix: &str) -> Result<Vec<RecordBatch>> {
    let conn = get_connection()?;
    let parquet_path = get_parquet_path();

    // <!-- SQL query to filter cities by prefix (case-insensitive) -->
    let query = format!(
        "SELECT * FROM read_parquet('{}') WHERE LOWER(city_name) LIKE LOWER('{}%') ORDER BY city_name LIMIT 20",
        parquet_path.to_string_lossy(),
        prefix.replace("'", "''")  // <!-- Escape single quotes for SQL safety -->
    );

    let rbs: Vec<RecordBatch> = conn.prepare(&query)?.query_arrow([])?.collect();

    Ok(rbs)
}

pub fn execute_query(query: &str) -> Result<Vec<RecordBatch>> {
    let conn = get_connection()?;
    let parquet_path = get_parquet_path();

    // <!-- Replace $PARQUET_PATH placeholder if present -->
    let final_query = query.replace("$PARQUET_PATH", &parquet_path.to_string_lossy());

    let rbs: Vec<RecordBatch> = conn.prepare(&final_query)?.query_arrow([])?.collect();

    Ok(rbs)
}

pub fn print_results(results: &[RecordBatch]) -> Result<()> {
    if results.is_empty() {
        println!("No results found.");
    } else {
        print_batches(results).map_err(|e| {
            duckdb::Error::FromSqlConversionFailure(
                0,
                duckdb::types::Type::Null,
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    e.to_string(),
                )),
            )
        })?;
    }
    Ok(())
}
