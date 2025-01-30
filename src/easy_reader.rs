use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use flate2::read::GzDecoder;
use ndarray::{s, Array2};
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;

const DELIMITER: u8 = b'\t';

/// Struct representing a reader for EEG data stored in `.easy` files.
///
/// This struct is responsible for parsing and storing the data from a `.easy` file,
/// which may include EEG signals, accelerometer data, and associated markers. The struct
/// loads the data from `.easy` and `.easy.gz` and optional `.info` files, provides methods for
/// accessing the data, and tracks relevant metadata about the file, including the
/// start date and number of channels.
#[derive(Debug)]
#[allow(dead_code)]
pub struct EasyReader {
    verbose: bool,
    /// Path to the `.easy` file being read.
    ///
    /// This is the full path to the `.easy` file that contains the EEG and accelerometer data.
    /// The file is parsed to extract the signals and metadata.
    filepath: String,

    /// Base name of the file without the extension.
    ///
    /// This is derived from the `filepath` and excludes the extension (e.g., `.easy` or `.easy.gz`).
    /// It is used for naming related files like the `.info` file.
    basename: String,

    /// The extension of the file (either "easy" or "easy.gz").
    ///
    /// This is used to identify the file type and determine how to process it.
    extension: String,

    /// Root of the file name (file path without extension).
    ///
    /// Used to construct the path for the associated `.info` file.
    filenameroot: String,

    /// Path to the associated `.info` file.
    ///
    /// If available, this file provides information about the electrode names and other metadata.
    infofilepath: String,

    /// Flag indicating whether accelerometer data is present.
    ///
    /// This flag is set to `true` if accelerometer data is found in the `.easy` file or the `.info` file.
    acc_data: bool,

    /// List of electrode names.
    ///
    /// If the `.info` file is available, this field will contain the names of the EEG channels (electrodes).
    /// If the `.info` file is not present, this will be populated with default channel names.
    electrodes: Vec<String>,

    /// Number of EEG channels.
    ///
    /// This represents the number of electrodes in the dataset (excluding accelerometer data).
    /// It is determined from the `.info` file or the `.easy` file.
    num_channels: Option<usize>,

    /// Start date of the EEG recording.
    ///
    /// This date is extracted from the first timestamp in the `.easy` file. It represents the
    /// time when the EEG recording began.
    eegstartdate: Option<String>,

    /// Array representing the time vector of the dataset in seconds.
    ///
    /// This array contains the time of each sample relative to the start of the recording.
    np_time: Option<Array2<f32>>,

    /// 2D array of EEG data.
    ///
    /// This is a 2D array where each row represents an EEG sample, and each column represents
    /// an individual channel (electrode). The data is in microvolts (uV).
    np_eeg: Option<Array2<f32>>,

    /// 2D array of stimulus data (optional).
    ///
    /// If present, this array contains stimulus information related to the EEG recording. It is typically used
    /// for event-marking or stimulus presentation data, but it may not always be available.
    np_stim: Option<Array2<f32>>,

    /// 2D array of accelerometer data.
    ///
    /// If accelerometer data is available, this array will contain the 3-axis accelerometer readings for each sample.
    /// The data represents the X, Y, and Z axes of the accelerometer. The array has shape `(num_samples, 3)`.
    np_acc: Option<Array2<f32>>,

    /// Array of markers associated with the EEG data.
    ///
    /// This array holds marker values that can represent events, triggers, or annotations
    /// in the EEG signal. Markers are typically used to mark specific moments in time during the recording.
    np_markers: Option<Array2<f32>>,

    /// Log of the events related to the processing of the `.easy` file.
    ///
    /// This is a collection of strings that logs important events, like the creation of the `EasyReader` instance
    /// and when key steps in the file processing were completed. This can be useful for debugging and tracking processing.
    log: Vec<String>,
}

impl EasyReader {
    /// Initializes a new `EasyReader` instance from the given file path.
    pub fn new(filepath: &str, verbose: bool) -> Result<Self> {
        if verbose {
            println!("Initializing in file path: {}", filepath);
        }

        let extension;
        let (filenameroot, basename) = if filepath.ends_with(".easy.gz") {
            extension = "easy.gz".to_string();
            let filenameroot = filepath.trim_end_matches(".gz");
            let basename = Path::new(filepath)
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .trim_end_matches(".gz")
                .to_string();
            (filenameroot.to_string(), basename)
        } else if filepath.ends_with(".easy") {
            extension = "easy".to_string();
            let filenameroot = filepath.trim_end_matches(".easy");
            let basename = Path::new(filepath)
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .trim_end_matches(".easy")
                .to_string();
            (filenameroot.to_string(), basename)
        } else {
            return Err(anyhow!("ERROR: Proposed file has wrong extension."));
        };

        let infofilepath = format!("{}.info", filenameroot);

        let mut reader = EasyReader {
            verbose,
            filepath: filepath.to_string(),
            basename,
            extension,
            filenameroot,
            infofilepath,
            acc_data: false,
            electrodes: Vec::new(),
            num_channels: None,
            eegstartdate: None,
            np_time: None,
            np_eeg: None,
            np_stim: None,
            np_acc: None,
            np_markers: None,
            log: vec![format!("capsule created: {}", Utc::now())],
        };

        // Try to read the info file
        reader.get_info()?;

        // Then read the easy data
        reader.get_l0_data()?;

        Ok(reader)
    }

    /// Reads and processes the `.info` file for metadata about channels and accelerometer data.
    fn get_info(&mut self) -> Result<()> {
        let file = File::open(&self.infofilepath);

        match file {
            Ok(file) => {
                let reader = BufReader::new(file);
                let mut electrodes = Vec::new();
                let mut acc_data = false;

                for line in reader.lines() {
                    let line = line.unwrap();
                    if line.contains("Channel ") {
                        let electrode = line.split_whitespace().last().unwrap().to_string();
                        electrodes.push(electrode);
                    }
                    if line.contains("Accelerometer data: ") {
                        acc_data = true;
                    }
                }

                self.electrodes = electrodes;
                self.acc_data = acc_data;
                self.num_channels = Some(self.electrodes.len());

                Ok(())
            }
            Err(_) => {
                // If no info file is found, read the .easy file to determine the number of channels
                self.read_easy_file_for_channels()
            }
        }
    }

    /// Reads the `.easy` file to determine the number of channels based on the file structure.
    fn read_easy_file_for_channels(&mut self) -> Result<()> {
        let reader = self.get_file_reader(&self.filepath)?;

        let mut rdr = csv::ReaderBuilder::new()
            .delimiter(DELIMITER)
            .has_headers(false)
            .from_reader(reader);

        // Read the first 5 lines to determine number of columns
        let mut header = rdr.records().take(5);
        let first_record = header.next().unwrap().unwrap();

        let num_columns = first_record.len();

        let num_channels = if [13, 25, 37].contains(&num_columns) {
            num_columns - 5
        } else if [10, 22, 34].contains(&num_columns) {
            num_columns - 2
        } else {
            return Err(anyhow!("Number of columns mismatch with expected values."));
        };

        self.num_channels = Some(num_channels);
        self.electrodes = (1..=num_channels).map(|x| format!("Ch{}", x)).collect();
        Ok(())
    }

    /// Reads and processes raw EEG and accelerometer data from the `.easy` file.
    ///
    /// This method reads the `.easy` file (or the data section of it), converts the EEG data
    /// into microvolts (uV), and extracts time, accelerometer, and marker data. It stores the
    /// resulting data in the struct's fields (e.g., `np_eeg`, `np_time`, `np_acc`, `np_markers`).
    /// It also logs key processing steps and ensures that the number of channels is consistent
    /// with the data found in the file.
    ///
    /// # Returns
    ///
    /// - `Ok(())` if the data was successfully read and processed.
    /// - `Err(String)` if there was an error reading or processing the file data. The error
    ///   string provides details about the failure (e.g., column mismatches or data format issues).
    ///
    /// # Details
    ///
    /// - The function expects the `.easy` file to have the following general format:
    ///   EEG data followed by accelerometer data (if available), markers, and timestamps.
    /// - The EEG data is divided by channels, and the accelerometer data (if present) consists
    ///   of three columns representing X, Y, and Z axes.

    fn get_l0_data(&mut self) -> Result<()> {
        let reader = self.get_file_reader(&self.filepath)?;
        let mut rdr = csv::ReaderBuilder::new()
            .delimiter(DELIMITER)
            .has_headers(false)
            .from_reader(reader);

        let mut records = rdr.records();
        let first_record = records.next().unwrap().unwrap();

        if self.verbose {
            println!("first_record - {first_record:?}");
        }

        let num_columns = first_record.len();

        let num_channels = if [13, 25, 37].contains(&num_columns) {
            num_columns - 5
        } else if [10, 22, 34].contains(&num_columns) {
            num_columns - 2
        } else {
            return Err(anyhow!("Number of columns mismatch with expected values."));
        };

        // Handle timestamp
        let timestamp = first_record[first_record.len() - 1].parse::<u64>().unwrap();
        if let Some(start_date) = DateTime::from_timestamp((timestamp / 1000) as i64, 0) {
            self.eegstartdate = Some(start_date.format("%Y-%m-%d %H:%M:%S").to_string());
        }

        if self.verbose {
            println!("Number of channels detected: {}", num_channels);
            println!(
                "First sample recorded: {}",
                self.eegstartdate.clone().unwrap()
            );
        }

        // Read the rest of the file into numpy-like data
        let mut eeg_data = Vec::new();
        let mut acc_data = Vec::new();
        let mut markers = Vec::new();

        for record in records {
            let record = record.unwrap();
            let eeg_values: Vec<f32> = record
                .iter()
                .take(num_channels)
                .map(|x| x.parse::<f32>().unwrap())
                .collect();
            let acc_values: Vec<f32> = record
                .iter()
                .skip(num_channels)
                .take(3)
                .map(|x| x.parse::<f32>().unwrap())
                .collect();
            let marker_value: f32 = record[num_channels + 3].parse().unwrap();

            eeg_data.push(eeg_values);
            acc_data.push(acc_values);
            markers.push(marker_value);
        }

        self.np_eeg = Some(
            Array2::from_shape_vec(
                (eeg_data.len(), num_channels),
                eeg_data.into_iter().flatten().collect(),
            )
            .unwrap(),
        );
        self.np_acc = Some(
            Array2::from_shape_vec(
                (acc_data.len(), 3),
                acc_data.into_iter().flatten().collect(),
            )
            .unwrap(),
        );
        self.np_markers = Some(Array2::from_shape_vec((markers.len(), 1), markers).unwrap());

        Ok(())
    }

    /// Helper function to get a reader for the file, whether it's gzipped or not.
    fn get_file_reader(&self, filepath: &str) -> Result<Box<dyn Read>> {
        if filepath.ends_with(".gz") {
            let file = File::open(filepath).map_err(|e| anyhow!(e.to_string()))?;
            let decoder = GzDecoder::new(file);
            Ok(Box::new(decoder))
        } else {
            let file = File::open(filepath).map_err(|e| anyhow!(e.to_string()))?;
            Ok(Box::new(file))
        }
    }

    /// Prints a summary of the `EasyReader` instance, displaying important metadata and previews of data.
    ///
    /// This function outputs the file path, base name, extension, number of channels, EEG start date,
    /// and any log entries related to the processing steps. It also prints the first few rows of the EEG,
    /// accelerometer, and markers data, if available. This method avoids printing the entire datasets.
    pub fn print_summary(&self) {
        // Print metadata
        println!("File Path: {}", self.filepath);
        println!("Base Name: {}", self.basename);
        println!("Extension: {}", self.extension);

        match &self.num_channels {
            Some(channels) => println!("Number of Channels: {}", channels),
            None => println!("Number of Channels: Not available"),
        }

        match &self.eegstartdate {
            Some(start_date) => println!("EEG Start Date: {}", start_date),
            None => println!("EEG Start Date: Not available"),
        }

        // Print a preview of EEG data (first 5 samples)
        match &self.np_eeg {
            Some(eeg) => {
                let total_samples = eeg.shape()[0];
                println!("\nEEG Data (First 5 of {total_samples} Samples):");
                let preview_count = total_samples.min(5); // Preview the first 5 samples or total samples if less than 5
                let preview: Vec<Vec<f32>> = eeg
                    .slice(s![..preview_count, ..]) // Get the first `preview_count` rows and all columns
                    .axis_iter(ndarray::Axis(0)) // Iterate over rows
                    .map(|row| row.to_owned().to_vec()) // Convert each row into a Vec<f32>
                    .collect(); // Collect all rows into a Vec<Vec<f32>>

                for (i, row) in preview.iter().enumerate() {
                    println!("Sample {}: {:?}", i + 1, row);
                }
                println!(
                    "Showing {} out of {} EEG samples.",
                    preview_count, total_samples
                );
            }
            None => println!("EEG Data: Not available"),
        }

        // Print a preview of accelerometer data (first 5 samples if available)
        match &self.np_acc {
            Some(acc) => {
                let total_samples = acc.shape()[0];
                println!("\nAccelerometer Data (First 5 of {total_samples} Samples):");
                let preview_count = total_samples.min(5); // Preview the first 5 samples or total samples if less than 5
                let preview: Vec<Vec<f32>> = acc
                    .slice(s![..preview_count, ..]) // Get the first `preview_count` rows and all columns
                    .axis_iter(ndarray::Axis(0)) // Iterate over rows
                    .map(|row| row.to_owned().to_vec()) // Convert each row into a Vec<f32>
                    .collect(); // Collect all rows into a Vec<Vec<f32>>

                for (i, row) in preview.iter().enumerate() {
                    println!("Sample {}: {:?}", i + 1, row);
                }
            }
            None => println!("Accelerometer Data: Not available"),
        }

        // Print a preview of markers (first 5 samples if available)
        match &self.np_markers {
            Some(markers) => {
                let total_samples = markers.shape()[0];
                println!("\nMarkers Data (First 5 of {total_samples} Samples):");
                let preview_count = total_samples.min(5); // Preview the first 5 samples or total samples if less than 5
                let (preview, _) = markers
                    .slice(s![..preview_count, ..]) // Get the first `preview_count` elements
                    .to_owned() // Copy the values from the slice
                    .into_raw_vec_and_offset(); // Convert it into a Vec<f32>

                for (i, marker) in preview.iter().enumerate() {
                    println!("Marker {}: {}", i + 1, marker);
                }
            }
            None => println!("Markers Data: Not available"),
        }

        // Print log entries
        println!("\nLog Entries:");
        for entry in &self.log {
            println!("- {}", entry);
        }
    }
}
