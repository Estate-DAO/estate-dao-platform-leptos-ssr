use anyhow::Result;
use duck_searcher::{get_parquet_path, print_results, search_cities_by_prefix, ConnectionPool};

fn main() -> Result<()> {
    // <!-- Example 1: Using the existing simple API (backward compatible) -->
    println!("=== Example 1: Using simple API ===\n");
    let search_prefix = "New";
    println!(
        "Searching for cities starting with '{}'...\n",
        search_prefix
    );
    let results = search_cities_by_prefix(search_prefix)?;
    print_results(&results)?;

    // <!-- Example 2: Creating and using a custom connection pool -->
    println!("\n=== Example 2: Using custom ConnectionPool ===\n");

    // <!-- Create a new in-memory connection pool with custom size -->
    let pool = ConnectionPool::new_memory(5)?;
    let parquet_path = get_parquet_path();

    // <!-- Example: Using execute method for batch SQL -->
    pool.execute(&format!(
        "CREATE VIEW city_view AS SELECT * FROM read_parquet('{}')",
        parquet_path.to_string_lossy()
    ))?;
    println!("Created city_view successfully");

    // <!-- Example: Query and get results as Arrow batches -->
    let conn = pool.get()?;
    let query =
        "SELECT * FROM city_view WHERE LOWER(city_name) LIKE 'lon%' ORDER BY city_name LIMIT 5";
    let results = conn.prepare(query)?.query_arrow([])?.collect::<Vec<_>>();
    println!("\nCities starting with 'Lon':");
    print_results(&results)?;

    // <!-- Example 3: Using get_json method -->
    println!("\n=== Example 3: Getting JSON output ===\n");
    let json_query =
        "SELECT city_name, country_code FROM city_view WHERE LOWER(city_name) LIKE 'par%' LIMIT 3";
    let json_bytes = pool.get_json(json_query)?;
    let json_str = String::from_utf8(json_bytes)?;
    println!("JSON output:\n{}", json_str);

    // <!-- Example 4: Creating a file-based persistent database -->
    println!("\n=== Example 4: File-based database ===\n");
    let file_pool = ConnectionPool::new_file("/tmp/cities.duckdb", 3)?;

    // <!-- Load data into the persistent database -->
    file_pool.execute(&format!(
        "CREATE TABLE IF NOT EXISTS cities AS SELECT * FROM read_parquet('{}')",
        parquet_path.to_string_lossy()
    ))?;
    println!("Created persistent cities table");

    // <!-- Query from persistent database -->
    let conn = file_pool.get()?;
    let query = "SELECT COUNT(*) as total FROM cities";
    let results = conn.prepare(query)?.query_arrow([])?.collect::<Vec<_>>();
    println!("\nTotal cities in database:");
    print_results(&results)?;

    Ok(())
}
