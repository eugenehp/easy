use anyhow::Result;
use chrono::{DateTime, MappedLocalTime, TimeZone, Utc};
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead};

/// Struct holding device information for EEG data.
#[derive(Debug)]
pub struct DeviceInfo {
    pub version: String,
    pub start_date: Option<DateTime<Utc>>,
    pub device_class: String,
    pub communication_type: String,
    pub device_id: String,
    pub software_version: String,
    pub firmware_version: String,
    pub os: String,
    pub sdcard_filename: String,
    pub additional_channel: String,
}

/// Struct for EEG settings including sampling rate, filters, and montage.
#[derive(Debug)]
pub struct EEGSettings {
    pub total_channels: usize,
    pub eeg_channels: usize,
    pub records: usize,
    pub sampling_rate: f32,
    pub configured_duration: u32,
    pub packets_lost: usize,
    pub line_filter: bool,
    pub fir_filter: bool,
    pub eog_correction: bool,
    pub reference_filter: bool,
    pub eeg_units: String,
    pub montage: HashMap<usize, String>,
    pub accelerometer: Option<AccelerometerData>,
}

/// Struct for accelerometer data.
#[derive(Debug)]
pub struct AccelerometerData {
    pub channels: usize,
    pub sampling_rate: f32,
    pub units: String,
}

/// Struct for trigger information in EEG data.
#[derive(Debug)]
pub struct TriggerInfo {
    pub triggers: HashMap<u32, String>,
}

/// Main struct representing EEG data, including device, settings, and trigger info.
#[derive(Debug)]
pub struct EEGData {
    pub device_info: DeviceInfo,
    pub eeg_settings: EEGSettings,
    pub trigger_info: TriggerInfo,
}

impl EEGData {
    /// Creates a new, empty EEGData struct.
    pub fn new() -> Self {
        EEGData {
            device_info: DeviceInfo {
                version: String::new(),
                start_date: None,
                device_class: String::new(),
                communication_type: String::new(),
                device_id: String::new(),
                software_version: String::new(),
                firmware_version: String::new(),
                os: String::new(),
                sdcard_filename: String::new(),
                additional_channel: String::new(),
            },
            eeg_settings: EEGSettings {
                total_channels: 0,
                eeg_channels: 0,
                records: 0,
                sampling_rate: 0.0,
                configured_duration: 0,
                packets_lost: 0,
                line_filter: false,
                fir_filter: false,
                eog_correction: false,
                reference_filter: false,
                eeg_units: String::new(),
                montage: HashMap::new(),
                accelerometer: None,
            },
            trigger_info: TriggerInfo {
                triggers: HashMap::new(),
            },
        }
    }

    /// Parses an EEG data file and returns an EEGData struct.
    pub fn parse_file(filename: &str) -> Result<Self> {
        let file = File::open(filename)?;
        let reader = io::BufReader::new(file);
        let mut data = EEGData::new();
        let mut current_section = None;

        for line in reader.lines() {
            let line = line?;

            if line.contains("Step Details") {
                current_section = Some("Step Details");
            } else if line.contains("EEG Settings") {
                current_section = Some("EEG Settings");
            } else if line.contains("Trigger information") {
                current_section = Some("Trigger information");
            }

            match current_section.as_deref() {
                Some("Step Details") => Self::parse_step_details(&line, &mut data),
                Some("EEG Settings") => Self::parse_eeg_settings(&line, &mut data),
                Some("Trigger information") => Self::parse_trigger_info(&line, &mut data),
                _ => continue,
            }
        }
        Ok(data)
    }

    /// Parses the 'Step Details' section of the file.
    fn parse_step_details(line: &str, data: &mut EEGData) {
        if line.contains("Info Version") {
            data.device_info.version = line.split(':').nth(1).unwrap_or("").trim().to_string();
        } else if line.contains("StartDate") {
            let timestamp: i64 = line
                .split(':')
                .nth(1)
                .unwrap_or("")
                .trim()
                .parse()
                .unwrap_or(0);

            data.device_info.start_date = match Utc.timestamp_millis_opt(timestamp) {
                MappedLocalTime::Single(dt) => Some(dt),
                MappedLocalTime::Ambiguous(early, _late) => Some(early),
                MappedLocalTime::None => None,
            }
        } else if line.contains("Device class") {
            data.device_info.device_class = line.split(':').nth(1).unwrap_or("").trim().to_string();
        } else if line.contains("Communication type") {
            data.device_info.communication_type =
                line.split(':').nth(1).unwrap_or("").trim().to_string();
        } else if line.contains("Device ID") {
            data.device_info.device_id = line.split(':').nth(1).unwrap_or("").trim().to_string();
        } else if line.contains("Software's version") {
            data.device_info.software_version =
                line.split(':').nth(1).unwrap_or("").trim().to_string();
        } else if line.contains("Firmware's version") {
            data.device_info.firmware_version =
                line.split(':').nth(1).unwrap_or("").trim().to_string();
        } else if line.contains("Operative system") {
            data.device_info.os = line.split(':').nth(1).unwrap_or("").trim().to_string();
        } else if line.contains("SDCard Filename") {
            data.device_info.sdcard_filename =
                line.split(':').nth(1).unwrap_or("").trim().to_string();
        } else if line.contains("Additional channel") {
            data.device_info.additional_channel =
                line.split(':').nth(1).unwrap_or("").trim().to_string();
        }
    }

    /// Parses the 'EEG Settings' section of the file.
    fn parse_eeg_settings(line: &str, data: &mut EEGData) {
        if line.contains("Total number of channels") {
            data.eeg_settings.total_channels = line
                .split(':')
                .nth(1)
                .unwrap_or("")
                .trim()
                .parse()
                .unwrap_or(0);
        } else if line.contains("Number of EEG channels") {
            data.eeg_settings.eeg_channels = line
                .split(':')
                .nth(1)
                .unwrap_or("")
                .trim()
                .parse()
                .unwrap_or(0);
        } else if line.contains("Number of records of EEG") {
            data.eeg_settings.records = line
                .split(':')
                .nth(1)
                .unwrap_or("")
                .trim()
                .parse()
                .unwrap_or(0);
        } else if line.contains("EEG sampling rate") {
            data.eeg_settings.sampling_rate = line
                .split(':')
                .nth(1)
                .unwrap_or("")
                .trim()
                .parse()
                .unwrap_or(0.0);
        } else if line.contains("EEG recording configured duration") {
            data.eeg_settings.configured_duration = line
                .split(':')
                .nth(1)
                .unwrap_or("")
                .trim()
                .parse()
                .unwrap_or(0);
        } else if line.contains("Number of packets lost") {
            data.eeg_settings.packets_lost = line
                .split(':')
                .nth(1)
                .unwrap_or("")
                .trim()
                .parse()
                .unwrap_or(0);
        } else if line.contains("Line filter status") {
            data.eeg_settings.line_filter = line.contains("ON");
        } else if line.contains("FIR filter status") {
            data.eeg_settings.fir_filter = line.contains("ON");
        } else if line.contains("EOG correction filter status") {
            data.eeg_settings.eog_correction = line.contains("ON");
        } else if line.contains("Reference filter status") {
            data.eeg_settings.reference_filter = line.contains("ON");
        } else if line.contains("EEG units") {
            data.eeg_settings.eeg_units = line.split(':').nth(1).unwrap_or("").trim().to_string();
        } else if line.contains("Accelerometer data") {
            if line.contains("ON") {
                let accelerometer = AccelerometerData {
                    channels: 3,
                    sampling_rate: 100.0,
                    units: "mm/s^2".to_string(),
                };
                data.eeg_settings.accelerometer = Some(accelerometer);
            }
        } else if line.contains("Channel") {
            let parts: Vec<&str> = line.split(':').collect();
            let channel_number = parts[0]
                .split_whitespace()
                .nth(1)
                .unwrap_or("")
                .trim()
                .parse()
                .unwrap_or(0);
            let electrode = parts[1].trim().to_string();
            data.eeg_settings.montage.insert(channel_number, electrode);
        }
    }

    /// Parses the 'Trigger information' section of the file.
    fn parse_trigger_info(line: &str, data: &mut EEGData) {
        // Skip header if found.
        if line.contains("Code") && line.contains("Description") {
            return;
        }

        // Parse trigger code and description.
        let parts: Vec<&str> = line.split_whitespace().collect();

        if parts.len() >= 2 {
            if let Ok(code) = parts[0].parse::<u32>() {
                let description = parts[1..].join(" ");
                data.trigger_info.triggers.insert(code, description);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    // Helper function to create a sample EEG file.
    fn create_sample_file() -> String {
        let file_content = r#"
        Step Details
        Info Version: 1.0
        StartDate: 1609459200000
        Device class: EEG
        Communication type: Bluetooth
        Device ID: 123456
        Software's version: 1.0.0
        Firmware's version: 1.0.1
        Operative system: Linux
        SDCard Filename: eegd_data.txt
        Additional channel: Channel_Extra

        EEG Settings
        Total number of channels: 8
        Number of EEG channels: 4
        Number of records of EEG: 1000
        EEG sampling rate: 250.0
        EEG recording configured duration: 3600
        Number of packets lost: 0
        Line filter status: ON
        FIR filter status: OFF
        EOG correction filter status: ON
        Reference filter status: OFF
        EEG units: µV
        Accelerometer data: ON
        Channel 1: Fp1
        Channel 2: Fp2
        Channel 3: F3
        Channel 4: F4

        Trigger information
        Code Description
        1 Start of EEG
        2 End of EEG
        "#;

        let filename = "sample_eeg_data.txt";
        std::fs::write(filename, file_content).unwrap();
        filename.to_string()
    }

    // Test for parsing EEG data file
    #[test]
    fn test_parse_file() {
        let filename = create_sample_file();
        let eeg_data = EEGData::parse_file(&filename).unwrap();

        // Test Device Info parsing
        assert_eq!(eeg_data.device_info.version, "1.0");
        assert_eq!(eeg_data.device_info.device_class, "EEG");
        assert_eq!(eeg_data.device_info.communication_type, "Bluetooth");
        assert_eq!(eeg_data.device_info.device_id, "123456");
        assert_eq!(eeg_data.device_info.software_version, "1.0.0");
        assert_eq!(eeg_data.device_info.firmware_version, "1.0.1");
        assert_eq!(eeg_data.device_info.os, "Linux");
        assert_eq!(eeg_data.device_info.sdcard_filename, "eegd_data.txt");
        assert_eq!(eeg_data.device_info.additional_channel, "Channel_Extra");

        // Test EEG Settings parsing
        assert_eq!(eeg_data.eeg_settings.total_channels, 8);
        assert_eq!(eeg_data.eeg_settings.eeg_channels, 4);
        assert_eq!(eeg_data.eeg_settings.records, 1000);
        assert_eq!(eeg_data.eeg_settings.sampling_rate, 250.0);
        assert_eq!(eeg_data.eeg_settings.configured_duration, 3600);
        assert_eq!(eeg_data.eeg_settings.packets_lost, 0);
        assert!(eeg_data.eeg_settings.line_filter);
        assert!(!eeg_data.eeg_settings.fir_filter);
        assert!(eeg_data.eeg_settings.eog_correction);
        assert!(!eeg_data.eeg_settings.reference_filter);
        assert_eq!(eeg_data.eeg_settings.eeg_units, "µV");

        // Test Accelerometer data
        assert!(eeg_data.eeg_settings.accelerometer.is_some());
        let accelerometer = eeg_data.eeg_settings.accelerometer.as_ref().unwrap();
        assert_eq!(accelerometer.channels, 3);
        assert_eq!(accelerometer.sampling_rate, 100.0);
        assert_eq!(accelerometer.units, "mm/s^2");

        // Test Montage parsing
        assert_eq!(
            eeg_data.eeg_settings.montage.get(&1),
            Some(&"Fp1".to_string())
        );
        assert_eq!(
            eeg_data.eeg_settings.montage.get(&2),
            Some(&"Fp2".to_string())
        );
        assert_eq!(
            eeg_data.eeg_settings.montage.get(&3),
            Some(&"F3".to_string())
        );
        assert_eq!(
            eeg_data.eeg_settings.montage.get(&4),
            Some(&"F4".to_string())
        );

        // Test Trigger Information parsing
        assert_eq!(eeg_data.trigger_info.triggers.len(), 2);
        assert_eq!(
            eeg_data.trigger_info.triggers.get(&1),
            Some(&"Start of EEG".to_string())
        );
        assert_eq!(
            eeg_data.trigger_info.triggers.get(&2),
            Some(&"End of EEG".to_string())
        );

        // Clean up the sample file
        std::fs::remove_file(filename).unwrap();
    }

    // Test parsing when the file is empty
    #[test]
    fn test_parse_empty_file() {
        let filename = "empty_file.txt";
        std::fs::write(filename, "").unwrap();

        let eeg_data = EEGData::parse_file(filename).unwrap();
        assert_eq!(eeg_data.device_info.version, "");
        assert_eq!(eeg_data.eeg_settings.total_channels, 0);
        assert_eq!(eeg_data.trigger_info.triggers.len(), 0);

        std::fs::remove_file(filename).unwrap();
    }

    // Test parsing when a field is missing (e.g., missing "StartDate" in the Step Details section)
    #[test]
    fn test_parse_missing_field() {
        let file_content = r#"
        Step Details
        Info Version: 1.0
        Device class: EEG
        Communication type: Bluetooth
        Device ID: 123456
        Software's version: 1.0.0
        Firmware's version: 1.0.1
        Operative system: Linux
        SDCard Filename: eegd_data.txt
        Additional channel: Channel_Extra

        EEG Settings
        Total number of channels: 8
        Number of EEG channels: 4
        Number of records of EEG: 1000
        EEG sampling rate: 250.0
        EEG recording configured duration: 3600
        Number of packets lost: 0
        Line filter status: ON
        FIR filter status: OFF
        EOG correction filter status: ON
        Reference filter status: OFF
        EEG units: µV
        Accelerometer data: ON

        Trigger information
        Code Description
        1 Start of EEG
        2 End of EEG
        "#;

        let filename = "missing_start_date.txt";
        std::fs::write(filename, file_content).unwrap();

        let eeg_data = EEGData::parse_file(filename).unwrap();
        assert!(eeg_data.device_info.start_date.is_none());

        std::fs::remove_file(filename).unwrap();
    }
}
