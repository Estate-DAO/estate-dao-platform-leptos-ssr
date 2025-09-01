use duck_searcher::{print_results, search_cities_by_prefix};
use duckdb::Result;

fn main() -> Result<()> {
    // <!-- Example usage: search for cities starting with "New" -->
    let search_prefix = "New";

    println!(
        "Searching for cities starting with '{}'...\n",
        search_prefix
    );

    let results = search_cities_by_prefix(search_prefix)?;
    print_results(&results)?;

    // <!-- Additional example searches -->
    println!("\n--- Another search example ---");
    println!("Searching for cities starting with 'Lon'...\n");
    let results2 = search_cities_by_prefix("Lon")?;
    print_results(&results2)?;

    // <!-- Example with a different prefix -->
    println!("\n--- Third search example ---");
    println!("Searching for cities starting with 'Par'...\n");
    let results3 = search_cities_by_prefix("Par")?;
    print_results(&results3)?;

    Ok(())
}
