use std::path::PathBuf;
use std::sync::OnceLock;

use anyhow::Result;
use arrow::{
    array::{Float64Array, StringArray},
    json::ArrayWriter,
    record_batch::RecordBatch,
    util::pretty::print_batches,
};
use duckdb::DuckdbConnectionManager;
use r2d2::{Pool, PooledConnection};
use serde::{Deserialize, Serialize};

type PooledConn = PooledConnection<DuckdbConnectionManager>;

static POOL: OnceLock<ConnectionPool> = OnceLock::new();

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CityEntry {
    pub city_code: String,
    pub city_name: String,
    pub country_name: String,
    pub country_code: String,
    pub image_url: String,
    pub latitude: f64,
    pub longitude: f64,
}

pub struct ConnectionPool {
    pool: Pool<DuckdbConnectionManager>,
    cities_table_initialized: std::sync::Arc<std::sync::Mutex<bool>>,
}

impl ConnectionPool {
    pub fn new_memory(pool_size: u32) -> Result<Self> {
        let manager = DuckdbConnectionManager::memory()?;
        let pool = Pool::builder().max_size(pool_size).build(manager)?;

        // <!-- Initialize parquet extension on a connection from pool -->
        let conn = pool.get()?;
        conn.execute_batch("INSTALL parquet; LOAD parquet;")?;
        drop(conn);

        Ok(Self {
            pool,
            cities_table_initialized: std::sync::Arc::new(std::sync::Mutex::new(false)),
        })
    }

    pub fn new_file(db_path: &str, pool_size: u32) -> Result<Self> {
        let manager = DuckdbConnectionManager::file(db_path)?;
        let pool = Pool::builder().max_size(pool_size).build(manager)?;

        // <!-- Initialize parquet extension on a connection from pool -->
        let conn = pool.get()?;
        conn.execute_batch("INSTALL parquet; LOAD parquet;")?;
        drop(conn);

        Ok(Self {
            pool,
            cities_table_initialized: std::sync::Arc::new(std::sync::Mutex::new(false)),
        })
    }

    pub fn get(&self) -> Result<PooledConn> {
        Ok(self.pool.get()?)
    }

    pub fn execute(&self, sql: &str) -> Result<()> {
        let conn = self.get()?;
        conn.execute_batch(sql)?;
        Ok(())
    }

    pub fn get_json(&self, sql: &str) -> Result<Vec<u8>> {
        let conn = self.get()?;
        let mut stmt = conn.prepare(sql)?;
        let arrow = stmt.query_arrow([])?;

        let buf = Vec::new();
        let mut writer = ArrayWriter::new(buf);
        for batch in arrow {
            writer.write(&batch)?;
        }
        writer.finish()?;
        Ok(writer.into_inner())
    }

    pub fn get_arrow(&self, sql: &str) -> Result<Vec<u8>> {
        let conn = self.get()?;
        let mut stmt = conn.prepare(sql)?;
        let arrow = stmt.query_arrow([])?;
        let schema = arrow.get_schema();

        let mut buffer: Vec<u8> = Vec::new();
        {
            let schema_ref = schema.as_ref();
            let mut writer = arrow::ipc::writer::FileWriter::try_new(&mut buffer, schema_ref)?;

            for batch in arrow {
                writer.write(&batch)?;
            }

            writer.finish()?;
        }

        Ok(buffer)
    }

    fn ensure_cities_table_loaded(&self) -> Result<()> {
        let mut initialized = self.cities_table_initialized.lock().unwrap();
        if *initialized {
            return Ok(());
        }

        let conn = self.get()?;
        let parquet_path = get_parquet_path();

        // <!-- Check if parquet file exists -->
        if !parquet_path.exists() {
            return Err(anyhow::anyhow!(
                "Cities parquet file not found at: {}",
                parquet_path.display()
            ));
        }

        // <!-- Create persistent cities table from parquet file -->
        conn.execute_batch("DROP TABLE IF EXISTS cities")?;
        conn.execute_batch(&format!(
            "CREATE TABLE cities AS SELECT * FROM read_parquet('{}')",
            parquet_path.to_string_lossy()
        ))?;

        *initialized = true;
        Ok(())
    }
}

pub fn get_parquet_path() -> PathBuf {
        PathBuf::from("city.parquet")
}

fn create_connection_pool() -> Result<ConnectionPool> {
    ConnectionPool::new_memory(20)
}

fn get_connection_pool() -> &'static ConnectionPool {
    POOL.get_or_init(|| create_connection_pool().expect("Failed to create connection pool"))
}

fn get_connection() -> Result<PooledConn> {
    let pool = get_connection_pool();
    pool.get()
}

/// Convert RecordBatch results to Vec<CityEntry>
pub fn convert_record_batches_to_cities(results: Vec<RecordBatch>) -> Result<Vec<CityEntry>> {
    let mut cities = Vec::new();

    for batch in results {
        let city_codes = batch
            .column_by_name("city_code")
            .ok_or_else(|| anyhow::anyhow!("Missing city_code column in search results"))?
            .as_any()
            .downcast_ref::<StringArray>()
            .ok_or_else(|| anyhow::anyhow!("Invalid city_code column type"))?;

        let city_names = batch
            .column_by_name("city_name")
            .ok_or_else(|| anyhow::anyhow!("Missing city_name column in search results"))?
            .as_any()
            .downcast_ref::<StringArray>()
            .ok_or_else(|| anyhow::anyhow!("Invalid city_name column type"))?;

        let country_names = batch
            .column_by_name("country_name")
            .ok_or_else(|| anyhow::anyhow!("Missing country_name column in search results"))?
            .as_any()
            .downcast_ref::<StringArray>()
            .ok_or_else(|| anyhow::anyhow!("Invalid country_name column type"))?;

        let country_codes = batch
            .column_by_name("country_code")
            .ok_or_else(|| anyhow::anyhow!("Missing country_code column in search results"))?
            .as_any()
            .downcast_ref::<StringArray>()
            .ok_or_else(|| anyhow::anyhow!("Invalid country_code column type"))?;

        let image_urls = batch
            .column_by_name("image_url")
            .ok_or_else(|| anyhow::anyhow!("Missing image_url column in search results"))?
            .as_any()
            .downcast_ref::<StringArray>()
            .ok_or_else(|| anyhow::anyhow!("Invalid image_url column type"))?;

        let latitudes = batch
            .column_by_name("latitude")
            .ok_or_else(|| anyhow::anyhow!("Missing latitude column in search results"))?
            .as_any()
            .downcast_ref::<Float64Array>()
            .ok_or_else(|| anyhow::anyhow!("Invalid latitude column type"))?;

        let longitudes = batch
            .column_by_name("longitude")
            .ok_or_else(|| anyhow::anyhow!("Missing longitude column in search results"))?
            .as_any()
            .downcast_ref::<Float64Array>()
            .ok_or_else(|| anyhow::anyhow!("Invalid longitude column type"))?;

        // Extract data from each row
        for i in 0..batch.num_rows() {
            let city_code = city_codes.value(i);
            let city_name = city_names.value(i);
            let country_name = country_names.value(i);
            let country_code = country_codes.value(i);
            let image_url = image_urls.value(i);
            let latitude = latitudes.value(i);
            let longitude = longitudes.value(i);

            cities.push(CityEntry {
                city_code: city_code.to_string(),
                city_name: city_name.to_string(),
                country_name: country_name.to_string(),
                country_code: country_code.to_string(),
                image_url: image_url.to_string(),
                latitude,
                longitude,
            });
        }
    }

    Ok(cities)
}

pub fn search_cities_by_prefix(prefix: &str) -> Result<Vec<RecordBatch>> {
    let pool = get_connection_pool();

    // <!-- Ensure the cities table is loaded from parquet -->
    pool.ensure_cities_table_loaded()?;

    let conn = pool.get()?;

    // <!-- SQL query to filter cities by prefix (case-insensitive) using table instead of reading parquet -->
    let query = format!(
        "SELECT * FROM cities WHERE LOWER(city_name) LIKE LOWER('{}%') ORDER BY city_name LIMIT 100",
        prefix.replace("'", "''") // <!-- Escape single quotes for SQL safety -->
    );

    let rbs: Vec<RecordBatch> = conn.prepare(&query)?.query_arrow([])?.collect();

    Ok(rbs)
}

/// Search cities by prefix and return CityEntry directly
pub fn search_cities_by_prefix_as_entries(prefix: &str) -> Result<Vec<CityEntry>> {
    let record_batches = search_cities_by_prefix(prefix)?;
    convert_record_batches_to_cities(record_batches)
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
        print_batches(results)?;
    }
    Ok(())
}

/// Write cities data to parquet file atomically
pub fn write_cities_to_parquet(cities: Vec<CityEntry>) -> Result<()> {
    use std::fs;
    use std::io::Write;

    let conn = get_connection()?;
    let parquet_path = get_parquet_path();

    // Create temporary files for atomic write
    let temp_parquet_path = parquet_path.with_extension("parquet.tmp");
    let temp_json_path = parquet_path.with_extension("json.tmp");

    // <!-- Ensure parent directory exists -->
    if let Some(parent) = parquet_path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Write cities to temporary JSON file
    let json_data = serde_json::to_string_pretty(&cities)?;
    let mut json_file = fs::File::create(&temp_json_path)?;
    json_file.write_all(json_data.as_bytes())?;
    json_file.sync_all()?;
    drop(json_file);

    // Create a temporary table and insert data from JSON file
    conn.execute_batch("DROP TABLE IF EXISTS temp_cities")?;
    conn.execute_batch(&format!(
        "CREATE TABLE temp_cities AS SELECT * FROM read_json_auto('{}')",
        temp_json_path.to_string_lossy()
    ))?;

    // Write to temporary parquet file
    conn.execute_batch(&format!(
        "COPY temp_cities TO '{}' (FORMAT PARQUET)",
        temp_parquet_path.to_string_lossy()
    ))?;

    // Clean up temporary table and JSON file
    conn.execute_batch("DROP TABLE temp_cities")?;
    fs::remove_file(&temp_json_path)?;

    // Atomic move: rename temp file to final location
    fs::rename(&temp_parquet_path, &parquet_path)?;

    Ok(())
}
