use std::fs::File;
use std::io::{self, BufRead};
use std::collections::HashMap;
use chrono::{DateTime, MappedLocalTime, TimeZone, Utc};
use anyhow::Result;

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

#[derive(Debug)]
pub struct AccelerometerData {
    pub channels: usize,
    pub sampling_rate: f32,
    pub units: String,
}

#[derive(Debug)]
pub struct TriggerInfo {
    pub triggers: HashMap<u32, String>,
}

#[derive(Debug)]
pub struct EEGData {
    pub device_info: DeviceInfo,
    pub eeg_settings: EEGSettings,
    pub trigger_info: TriggerInfo,
}

impl EEGData {
    pub fn new() -> Self {
        EEGData {
            device_info: DeviceInfo {
                version: String::new(),
                start_date: Some(Utc::now()),
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

    fn parse_step_details(line: &str, data: &mut EEGData) {
        if line.contains("Info Version") {
            data.device_info.version = line.split(':').nth(1).unwrap_or("").trim().to_string();
        } else if line.contains("StartDate") {
            let timestamp: i64 = line.split(':').nth(1).unwrap_or("").trim().parse().unwrap_or(0);

            data.device_info.start_date = match Utc.timestamp_millis_opt(timestamp) {
                MappedLocalTime::Single(dt) => Some(dt),
                MappedLocalTime::Ambiguous(early, _late) => Some(early),
                MappedLocalTime::None => None,
            }
        } else if line.contains("Device class") {
            data.device_info.device_class = line.split(':').nth(1).unwrap_or("").trim().to_string();
        } else if line.contains("Communication type") {
            data.device_info.communication_type = line.split(':').nth(1).unwrap_or("").trim().to_string();
        } else if line.contains("Device ID") {
            data.device_info.device_id = line.split(':').nth(1).unwrap_or("").trim().to_string();
        } else if line.contains("Software's version") {
            data.device_info.software_version = line.split(':').nth(1).unwrap_or("").trim().to_string();
        } else if line.contains("Firmware's version") {
            data.device_info.firmware_version = line.split(':').nth(1).unwrap_or("").trim().to_string();
        } else if line.contains("Operative system") {
            data.device_info.os = line.split(':').nth(1).unwrap_or("").trim().to_string();
        } else if line.contains("SDCard Filename") {
            data.device_info.sdcard_filename = line.split(':').nth(1).unwrap_or("").trim().to_string();
        } else if line.contains("Additional channel") {
            data.device_info.additional_channel = line.split(':').nth(1).unwrap_or("").trim().to_string();
        }
    }

    fn parse_eeg_settings(line: &str, data: &mut EEGData) {
        if line.contains("Total number of channels") {
            data.eeg_settings.total_channels = line.split(':').nth(1).unwrap_or("").trim().parse().unwrap_or(0);
        } else if line.contains("Number of EEG channels") {
            data.eeg_settings.eeg_channels = line.split(':').nth(1).unwrap_or("").trim().parse().unwrap_or(0);
        } else if line.contains("Number of records of EEG") {
            data.eeg_settings.records = line.split(':').nth(1).unwrap_or("").trim().parse().unwrap_or(0);
        } else if line.contains("EEG sampling rate") {
            data.eeg_settings.sampling_rate = line.split(':').nth(1).unwrap_or("").trim().parse().unwrap_or(0.0);
        } else if line.contains("EEG recording configured duration") {
            data.eeg_settings.configured_duration = line.split(':').nth(1).unwrap_or("").trim().parse().unwrap_or(0);
        } else if line.contains("Number of packets lost") {
            data.eeg_settings.packets_lost = line.split(':').nth(1).unwrap_or("").trim().parse().unwrap_or(0);
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
            let channel_number = parts[0].split_whitespace().nth(1).unwrap_or("").trim().parse().unwrap_or(0);
            let electrode = parts[1].trim().to_string();
            data.eeg_settings.montage.insert(channel_number, electrode);
        }
    }

    fn parse_trigger_info(line: &str, data: &mut EEGData) {
        // We will check if the line contains trigger information, 
        // specifically looking for the "Code" and "Description" keywords.
        if line.contains("Code") && line.contains("Description") {
            // This line might be the header, so we can skip it.
            return;
        }
    
        // If the line contains a trigger code and description, we will try to extract them.
        let parts: Vec<&str> = line.split_whitespace().collect();
    
        if parts.len() >= 2 {
            // The first part should be the trigger code, the rest should be the description.
            if let Ok(code) = parts[0].parse::<u32>() {
                // Join the rest of the parts as the description
                let description = parts[1..].join(" ");
                // Insert the code and description into the triggers HashMap
                data.trigger_info.triggers.insert(code, description);
            }
        }
    }
}