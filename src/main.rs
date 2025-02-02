use anyhow::Result;
use easy_rs::{easy_reader::EasyReader, info::EEGData};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let filename = "data/Example01.info";
    let data = EEGData::parse_file(filename)?;
    println!("{data:#?}");

    let filename = "data/Example01.easy";
    // let filename = "data/Example01.easy.gz";
    let mut reader = EasyReader::new(filename, false)?;
    // Try to read the info file
    // reader.get_info()?;

    // Then read the easy data, all at once
    // reader.get_l0_data()?;

    // reader.print_summary();
    // println!("{reader:#?}");

    // a streaming example
    reader.get_l0_data_streaming(Some(1), |eeg_chunk, acc_chunk, markers_chunk| {
        // Process the chunk, for example, you could print the first few samples or store them
        println!("Processing chunk of size: {}", eeg_chunk.len());
        println!("First EEG sample: {:?}", eeg_chunk.first());
        println!("First Acc sample: {:?}", acc_chunk.first());
        println!("First Marker: {:?}", markers_chunk.first());
    })?;

    Ok(())
}
