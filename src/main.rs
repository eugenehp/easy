use easy::info::EEGData;
use anyhow::Result;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let filename = "data/Example01.info";
    let data = EEGData::parse_file(filename)?;

    println!("{data:#?}");

    Ok(())
}