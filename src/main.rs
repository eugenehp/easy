use anyhow::Result;
use easy::{easy_reader::EasyReader, info::EEGData};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let filename = "data/Example01.info";
    let data = EEGData::parse_file(filename)?;
    println!("{data:#?}");

    let filename = "data/Example01.easy";
    let reader = EasyReader::new(filename, false)?;

    reader.print_summary();
    println!("{reader:#?}");

    Ok(())
}
