use duck_searcher::{execute_query, get_parquet_path, print_results};
use std::io::{self, Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("DuckDB REPL for City Database");
    println!("=============================");
    println!("Type SQL queries to explore the city parquet file.");
    println!("Use $PARQUET_PATH as placeholder for the parquet file path.");
    println!("Type 'help' for examples, 'exit' or 'quit' to exit.\n");

    let parquet_path = get_parquet_path();
    println!("Parquet file: {}\n", parquet_path.display());

    loop {
        print!("duckdb> ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        match input.to_lowercase().as_str() {
            "exit" | "quit" => {
                println!("Goodbye!");
                break;
            }
            "help" => {
                print_help();
            }
            "" => continue,
            _ => match execute_query(input) {
                Ok(results) => {
                    print_results(&results)?;
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                }
            },
        }
        println!();
    }

    Ok(())
}

fn print_help() {
    println!("\nExample queries:");
    println!("  SELECT * FROM read_parquet('$PARQUET_PATH') LIMIT 5;");
    println!("  SELECT * FROM read_parquet('$PARQUET_PATH') WHERE LOWER(city_name) LIKE LOWER('A%') ORDER BY city_name LIMIT 100;");
    println!("  SELECT * FROM read_parquet('$PARQUET_PATH') where LOWER(city_name) = LOWER('a') ORDER BY city_name LIMIT 100;");
    println!("  SELECT city_name, country_name FROM read_parquet('$PARQUET_PATH') WHERE city_name LIKE 'A%' LIMIT 5;");
    println!("  SELECT COUNT(*) as total_cities FROM read_parquet('$PARQUET_PATH');");
    println!("  SELECT country_name, COUNT(*) as city_count FROM read_parquet('$PARQUET_PATH') GROUP BY country_name ORDER BY city_count DESC LIMIT 10;");
    println!(
        "  SELECT DISTINCT country_name FROM read_parquet('$PARQUET_PATH') ORDER BY country_name;"
    );
    println!("\nSpecial commands:");
    println!("  help  - Show this help message");
    println!("  exit  - Exit the REPL");
    println!("  quit  - Exit the REPL");
}
