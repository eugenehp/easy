use anyhow::Result;
use easy::{easy_reader, info::EEGData};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // let filename = "data/Example01.info";
    // let data = EEGData::parse_file(filename)?;

    let filename = "data/Example01.easy";
    let reader = easy_reader::EasyReader::new(filename)?;

    println!("{reader:#?}");

    Ok(())
}
