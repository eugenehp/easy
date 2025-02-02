
# Easy - Rust Library for EEG and Accelerometer Data Processing

`easy` is a Rust library designed for reading and processing EEG and accelerometer data stored in `.easy` and `.easy.gz` files. It provides tools for extracting and parsing EEG signals, accelerometer data, timestamps, and event markers, making it suitable for processing neuroscience data in Rust.

## Features

- **EEG Data**: Load and process EEG data from `.easy` files.
- **Accelerometer Data**: Extract and process accelerometer data (if available).
- **Markers**: Handle event markers associated with the EEG data.
- **Flexible File Parsing**: Supports both `.easy` and `.easy.gz` file formats.
- **Metadata Extraction**: Read metadata from associated `.info` files, including electrode names, number of channels, and recording start time.

## Installation

To add the `easy-rs` library to your project, include it as a dependency in your `Cargo.toml`:

```toml
[dependencies]
easy-rs = "0.0.3"
```

Alternatively, you can clone the repository directly and build from source:

```bash
git clone https://github.com/eugenehp/easy.git
cd easy
cargo build
```

## Usage

### Basic Example

Here’s a basic example of how to use the `EasyReader` struct to load and process an `.easy` file:

```rust
use anyhow::Result;
use easy_rs::{easy_reader::EasyReader, info::EEGData};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let filename = "data/Example01.info";
    let data = EEGData::parse_file(filename)?;
    println!("{data:#?}");

    let filename = "data/Example01.easy";
    // let filename = "data/Example01.easy.gz";
    let reader = EasyReader::new(filename, false)?;

    reader.print_summary();
    // println!("{reader:#?}");

    Ok(())
}

```

### Accessing Processed Data

Once the file is loaded, you can access the processed data like this:

```rust
let eeg_data = reader.np_eeg.as_ref().unwrap();
let accelerometer_data = reader.np_acc.as_ref().unwrap();
let markers = reader.np_markers.as_ref().unwrap();

// Example: Print the first 5 EEG samples
println!("First 5 EEG samples: {:?}", eeg_data.slice(s![..5, ..]));
```

### Available Methods

- `EasyReader::new(filepath: &str, verbose: bool)`: Initializes the reader for a given `.easy` file.
- `EasyReader::get_info()`: Reads metadata from the `.info` file if available.
- `EasyReader::get_l0_data()`: Reads and processes the EEG and accelerometer data from the `.easy` file.
- `EasyReader::print_summary()`: Prints a summary of the loaded data, including EEG channels, start time, and a preview of the data.

## File Formats

- **.easy**: The main file format that contains the EEG and possibly accelerometer data.
- **.easy.gz**: A compressed version of the `.easy` file.
- **.info**: An optional metadata file that contains information about the EEG channels and accelerometer data (if applicable).

## Contributing

If you'd like to contribute to the project, feel free to open an issue or submit a pull request. Here are a few guidelines:

1. Fork the repository.
2. Create a new branch for your feature or fix.
3. Write tests to cover your changes.
4. Ensure all tests pass by running `cargo test`.
5. Submit a pull request with a clear description of your changes.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Contact

For any questions or feedback, feel free to open an issue on the [GitHub repository](https://github.com/eugenehp/easy).

## Thank you

Inspired by [NEPy](https://github.com/Neuroelectrics/NEPy)

## Copyright

2025, Eugene Hauptmann