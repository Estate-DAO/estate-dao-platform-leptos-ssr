use anyhow::Result;
use arrow::json::ReaderBuilder;
use parquet::arrow::ArrowWriter;
use std::env;
use std::fs::File;
use std::io::BufReader;
use std::sync::Arc;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() != 3 {
        eprintln!("Usage: {} <input.json> <output.parquet>", args[0]);
        std::process::exit(1);
    }
    
    let input_path = &args[1];
    let output_path = &args[2];
    
    // Read JSON file
    let file = File::open(input_path)?;
    let reader = BufReader::new(file);
    
    // Create Arrow reader from JSON
    let mut json_reader = ReaderBuilder::new()
        .infer_schema(Some(1000))
        .build(reader)?;
    
    // Read all batches
    let mut batches = Vec::new();
    while let Some(batch) = json_reader.next() {
        batches.push(batch?);
    }
    
    if batches.is_empty() {
        anyhow::bail!("No data found in JSON file");
    }
    
    // Write to Parquet
    let schema = batches[0].schema();
    let output_file = File::create(output_path)?;
    let mut writer = ArrowWriter::try_new(output_file, Arc::clone(&schema), None)?;
    
    for batch in batches {
        writer.write(&batch)?;
    }
    writer.close()?;
    
    println!("Successfully converted {} to {}", input_path, output_path);
    Ok(())
}