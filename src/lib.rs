pub mod config;
pub mod ser;
use std::{
    fs::{read_dir, remove_file, File},
    io::{BufReader, BufWriter, Write},
    path::PathBuf,
};

use chrono::{DateTime, Datelike, Duration, TimeZone, Utc};
use config::Entry;
use ical::{
    parser::{
        ical::component::{IcalCalendar, IcalEvent},
        vcard::component::VcardContact,
    },
    property::Property,
};
use thiserror::Error;

use crate::ser::calendar_to_string;

#[derive(Debug, PartialEq)]
struct ExtractedDate {
    year: Option<i32>,
    month: u32,
    day: u32,
}

#[derive(Error, Debug)]
pub enum EventExtractorError {
    #[error("property \"{}\" was not found", .0)]
    PropertyNotFound(String),
    #[error("value of property \"{}\" not found", .0)]
    PropertyValueNotFound(String),
    #[error("date extraction failed: {}", .0)]
    DateExtractionFailed(String),
    #[error("parsing the {} value \"{}\" failed", .0, .1)]
    ParseDateFailed(String, String),
    #[error("unexpected date format")]
    UnexpectedDateFormat,
    #[error("serialization error")]
    SerializationError(#[from] ser::SerializationError),
    #[error("configuration error")]
    ConfigError(#[from] config::ConfigError),
    #[error("std::io error")]
    StdIoError(#[from] std::io::Error),
    #[error("ical::parser parse error")]
    IcalParseError(#[from] ical::parser::ParserError),
}

pub fn process_entry(config_entry: &Entry) -> Result<(), EventExtractorError> {
    if config_entry.remove_files {
        // remove existing files
        log::info!("removing exiting files");
        for entry in read_dir(&config_entry.output)? {
            let path = entry?.path();

            if path.is_file()
                && match path.extension() {
                    Some(extension) => extension == "ics",
                    None => false,
                }
            {
                remove_file(path)?
            }
        }
    }

    let current_year = Utc::now().year();
    let years: Vec<i32> = vec![-1, 0, 1, 2]
        .iter()
        .map(|offset| current_year + offset)
        .collect();
    log::info!(
        "generating entries for years: {}",
        years
            .iter()
            .map(|elem| elem.to_string())
            .collect::<Vec<String>>()
            .join(", ")
    );

    // create new files
    for entry in read_dir(&config_entry.input)? {
        let path = entry?.path();

        log::debug!(
            "found entry \"{}\", is file: {}, ends with .vcf: {}",
            path.to_string_lossy(),
            path.is_file(),
            match path.extension() {
                Some(extension) => extension == "vcf",
                None => false,
            }
        );

        if path.is_file()
            && match path.extension() {
                Some(extension) => extension == "vcf",
                None => false,
            }
        {
            log::info!("processing file \"{}\"", path.to_string_lossy());

            let buf = BufReader::new(File::open(&path).unwrap());
            let reader = ical::VcardParser::new(buf);

            for vcard in reader {
                let contact = vcard?;
                for event in convert(&contact, &years)? {
                    let uid = event
                        .properties
                        .iter()
                        .find(|&elem| elem.name == "UID")
                        .ok_or(EventExtractorError::PropertyNotFound("UID".into()))?
                        .value
                        .as_ref()
                        .ok_or(EventExtractorError::PropertyValueNotFound("UID".into()))?
                        .clone();
                    let mut filename = PathBuf::from(&config_entry.output);
                    filename.push(format!("{}.ics", uid));

                    let cal = IcalCalendar {
                        properties: vec![
                            Property {
                                name: "VERSION".into(),
                                value: Some("2.0".into()),
                                ..Default::default()
                            },
                            Property {
                                name: "PRODID".into(),
                                value: Some("event-extractor//hochreiner.net".into()),
                                ..Default::default()
                            },
                        ],
                        events: vec![event],
                        ..Default::default()
                    };

                    let mut writer = BufWriter::new(File::create(filename)?);

                    writer.write_all(calendar_to_string(&cal)?.as_bytes())?;
                }
            }
        }
    }

    Ok(())
}

pub fn convert(
    contact: &VcardContact,
    years: &Vec<i32>,
) -> Result<Vec<IcalEvent>, EventExtractorError> {
    let mut fn_prop = None;
    let mut bday_prop = None;
    let mut uid_prop = None;
    let timestamp = Utc::now();

    for prop in &contact.properties {
        match prop.name.as_str() {
            "FN" => fn_prop = Some(prop),
            "BDAY" => bday_prop = Some(prop),
            "UID" => uid_prop = Some(prop),
            _ => {}
        }
    }

    match (fn_prop, uid_prop, bday_prop) {
        (None, _, _) => Err(EventExtractorError::PropertyNotFound("FN".to_string())),
        (_, None, _) => Err(EventExtractorError::PropertyNotFound("UID".to_string())),
        (Some(fn_prop), Some(uid_prop), Some(bday_prop)) => generate_events_for_years(
            fn_prop,
            uid_prop,
            &ExtractedDate::try_from(bday_prop)?,
            years,
            &timestamp,
        ),
        (_, _, None) => Ok(Vec::new()),
    }
}

fn generate_events_for_years(
    fn_prop: &Property,
    uid_prop: &Property,
    date: &ExtractedDate,
    years: &Vec<i32>,
    timestamp: &DateTime<Utc>,
) -> Result<Vec<IcalEvent>, EventExtractorError> {
    let uid = uid_prop
        .value
        .as_ref()
        .ok_or(EventExtractorError::PropertyValueNotFound(
            "UID".to_string(),
        ))?;
    let fn_value = fn_prop
        .value
        .as_ref()
        .ok_or(EventExtractorError::PropertyValueNotFound("FN".into()))?;

    years
        .iter()
        .map(|year| {
            let mut event = IcalEvent::new();
            let start_date = Utc
                .with_ymd_and_hms(*year, date.month, date.day, 0, 0, 0)
                .earliest()
                .ok_or(EventExtractorError::UnexpectedDateFormat)?;
            let end_date = start_date + Duration::days(1);

            event.properties.append(&mut vec![
                Property {
                    name: "UID".into(),
                    params: None,
                    value: Some(format!("{}_bday_{}", uid, year)),
                },
                Property {
                    name: "DTSTAMP".into(),
                    params: None,
                    value: Some(timestamp.format("%Y%m%dT%H%M%SZ").to_string()),
                },
                Property {
                    name: "STATUS".to_string(),
                    params: None,
                    value: Some("CONFIRMED".into()),
                },
                Property {
                    name: "TRANSP".into(),
                    params: None,
                    value: Some("TRANSPARENT".into()),
                },
                Property {
                    name: "DTSTART".into(),
                    params: Some(vec![("VALUE".into(), vec!["DATE".into()])]),
                    value: Some(start_date.format("%Y%m%d").to_string()),
                },
                Property {
                    name: "DTEND".into(),
                    params: Some(vec![("VALUE".into(), vec!["DATE".into()])]),
                    value: Some(end_date.format("%Y%m%d").to_string()),
                },
                Property {
                    name: "SUMMARY".into(),
                    params: fn_prop.params.clone(),
                    value: Some(match date.year {
                        Some(date_year) => format!("Birthday: {} ({})", fn_value, year - date_year),
                        None => format!("Birthday: {}", fn_value),
                    }),
                },
            ]);

            Ok(event)
        })
        .collect()
}

impl TryFrom<&Property> for ExtractedDate {
    type Error = EventExtractorError;

    fn try_from(property: &Property) -> Result<Self, Self::Error> {
        match property
            .params
            .as_ref()
            .ok_or(EventExtractorError::DateExtractionFailed(
                "no parameters found".to_string(),
            ))?
            .iter()
            .find(|&(key, _)| key == "VALUE")
        {
            Some((_, param_values)) => {
                if param_values.len() != 1 {
                    return Err(EventExtractorError::DateExtractionFailed(
                        "value type not unique".to_string(),
                    ));
                }

                let param_value =
                    param_values
                        .get(0)
                        .ok_or(EventExtractorError::DateExtractionFailed(
                            "value type not found".to_string(),
                        ))?;

                if param_value != "DATE" {
                    return Err(EventExtractorError::DateExtractionFailed(format!(
                        "expected value type \"DATE\" found \"{}\"",
                        param_value
                    )));
                }

                let property_value =
                    property
                        .value
                        .as_ref()
                        .ok_or(EventExtractorError::DateExtractionFailed(
                            "no date value found".to_string(),
                        ))?;

                match (property_value.len(), property_value.starts_with("--")) {
                    (8, false) => Ok(ExtractedDate {
                        year: Some(property_value[0..4].parse().map_err(|_| {
                            EventExtractorError::ParseDateFailed(
                                "year".into(),
                                property_value[0..4].to_string(),
                            )
                        })?),
                        month: property_value[4..6].parse().map_err(|_| {
                            EventExtractorError::ParseDateFailed(
                                "month".into(),
                                property_value[4..6].to_string(),
                            )
                        })?,
                        day: property_value[6..8].parse().map_err(|_| {
                            EventExtractorError::ParseDateFailed(
                                "day".to_string(),
                                property_value[6..8].to_string(),
                            )
                        })?,
                    }),
                    (6, true) => Ok(ExtractedDate {
                        year: None,
                        month: property_value[2..4].parse().map_err(|_| {
                            EventExtractorError::ParseDateFailed(
                                "month".into(),
                                property_value[4..6].to_string(),
                            )
                        })?,
                        day: property_value[4..6].parse().map_err(|_| {
                            EventExtractorError::ParseDateFailed(
                                "day".to_string(),
                                property_value[6..8].to_string(),
                            )
                        })?,
                    }),
                    (_, _) => Err(EventExtractorError::UnexpectedDateFormat),
                }
            }
            None => Err(EventExtractorError::DateExtractionFailed(
                "could not determine value type".to_string(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracted_date_1() {
        assert_eq!(
            &ExtractedDate::try_from(&Property {
                name: "BDAY".to_string(),
                value: Some("--0214".to_string()),
                params: Some(vec![("VALUE".to_string(), vec!["DATE".to_string()])])
            })
            .unwrap(),
            &ExtractedDate {
                year: None,
                month: 2,
                day: 14
            }
        )
    }

    #[test]
    fn extracted_date_2() {
        assert_eq!(
            ExtractedDate::try_from(&Property {
                name: "BDAY".to_string(),
                value: Some("19961023".to_string()),
                params: Some(vec![("VALUE".to_string(), vec!["DATE".to_string()])])
            })
            .unwrap(),
            ExtractedDate {
                year: Some(1996),
                month: 10,
                day: 23
            }
        )
    }

    #[test]
    fn generate_events_for_years_1() {
        let timestamp = Utc::now();

        assert_eq!(
            format!(
                "{:?}",
                generate_events_for_years(
                    &Property {
                        name: "FN".into(),
                        params: None,
                        value: Some("Test Person".into())
                    },
                    &Property {
                        name: "UID".into(),
                        params: None,
                        value: Some("test_uid".into())
                    },
                    &ExtractedDate {
                        year: Some(1990),
                        month: 10,
                        day: 5
                    },
                    &vec![2000, 2001],
                    &timestamp
                )
                .unwrap()
            ),
            format!(
                "{:?}",
                vec![
                    IcalEvent {
                        alarms: Vec::new(),
                        properties: vec![
                            Property {
                                name: "UID".into(),
                                params: None,
                                value: Some("test_uid_bday_2000".into()),
                            },
                            Property {
                                name: "DTSTAMP".into(),
                                params: None,
                                value: Some(timestamp.format("%Y%m%dT%H%M%SZ").to_string()),
                            },
                            Property {
                                name: "STATUS".to_string(),
                                params: None,
                                value: Some("CONFIRMED".into()),
                            },
                            Property {
                                name: "TRANSP".into(),
                                params: None,
                                value: Some("TRANSPARENT".into()),
                            },
                            Property {
                                name: "DTSTART".into(),
                                params: Some(vec![("VALUE".into(), vec!["DATE".into()])]),
                                value: Some("20001005".into()),
                            },
                            Property {
                                name: "DTEND".into(),
                                params: Some(vec![("VALUE".into(), vec!["DATE".into()])]),
                                value: Some("20001006".into()),
                            },
                            Property {
                                name: "SUMMARY".into(),
                                params: None,
                                value: Some("Birthday: Test Person (10)".into()),
                            },
                        ]
                    },
                    IcalEvent {
                        alarms: Vec::new(),
                        properties: vec![
                            Property {
                                name: "UID".into(),
                                params: None,
                                value: Some("test_uid_bday_2001".into()),
                            },
                            Property {
                                name: "DTSTAMP".into(),
                                params: None,
                                value: Some(timestamp.format("%Y%m%dT%H%M%SZ").to_string()),
                            },
                            Property {
                                name: "STATUS".to_string(),
                                params: None,
                                value: Some("CONFIRMED".into()),
                            },
                            Property {
                                name: "TRANSP".into(),
                                params: None,
                                value: Some("TRANSPARENT".into()),
                            },
                            Property {
                                name: "DTSTART".into(),
                                params: Some(vec![("VALUE".into(), vec!["DATE".into()])]),
                                value: Some("20011005".into()),
                            },
                            Property {
                                name: "DTEND".into(),
                                params: Some(vec![("VALUE".into(), vec!["DATE".into()])]),
                                value: Some("20011006".into()),
                            },
                            Property {
                                name: "SUMMARY".into(),
                                params: None,
                                value: Some("Birthday: Test Person (11)".into()),
                            },
                        ]
                    }
                ]
            )
        )
    }

    #[test]
    fn generate_events_for_years_2() {
        let timestamp = Utc::now();

        assert_eq!(
            format!(
                "{:?}",
                generate_events_for_years(
                    &Property {
                        name: "FN".into(),
                        params: None,
                        value: Some("Test Person".into())
                    },
                    &Property {
                        name: "UID".into(),
                        params: None,
                        value: Some("test_uid".into())
                    },
                    &ExtractedDate {
                        year: None,
                        month: 10,
                        day: 5
                    },
                    &vec![2012, 2013],
                    &timestamp
                )
                .unwrap()
            ),
            format!(
                "{:?}",
                vec![
                    IcalEvent {
                        alarms: Vec::new(),
                        properties: vec![
                            Property {
                                name: "UID".into(),
                                params: None,
                                value: Some("test_uid_bday_2012".into()),
                            },
                            Property {
                                name: "DTSTAMP".into(),
                                params: None,
                                value: Some(timestamp.format("%Y%m%dT%H%M%SZ").to_string()),
                            },
                            Property {
                                name: "STATUS".to_string(),
                                params: None,
                                value: Some("CONFIRMED".into()),
                            },
                            Property {
                                name: "TRANSP".into(),
                                params: None,
                                value: Some("TRANSPARENT".into()),
                            },
                            Property {
                                name: "DTSTART".into(),
                                params: Some(vec![("VALUE".into(), vec!["DATE".into()])]),
                                value: Some("20121005".into()),
                            },
                            Property {
                                name: "DTEND".into(),
                                params: Some(vec![("VALUE".into(), vec!["DATE".into()])]),
                                value: Some("20121006".into()),
                            },
                            Property {
                                name: "SUMMARY".into(),
                                params: None,
                                value: Some("Birthday: Test Person".into()),
                            },
                        ]
                    },
                    IcalEvent {
                        alarms: Vec::new(),
                        properties: vec![
                            Property {
                                name: "UID".into(),
                                params: None,
                                value: Some("test_uid_bday_2013".into()),
                            },
                            Property {
                                name: "DTSTAMP".into(),
                                params: None,
                                value: Some(timestamp.format("%Y%m%dT%H%M%SZ").to_string()),
                            },
                            Property {
                                name: "STATUS".to_string(),
                                params: None,
                                value: Some("CONFIRMED".into()),
                            },
                            Property {
                                name: "TRANSP".into(),
                                params: None,
                                value: Some("TRANSPARENT".into()),
                            },
                            Property {
                                name: "DTSTART".into(),
                                params: Some(vec![("VALUE".into(), vec!["DATE".into()])]),
                                value: Some("20131005".into()),
                            },
                            Property {
                                name: "DTEND".into(),
                                params: Some(vec![("VALUE".into(), vec!["DATE".into()])]),
                                value: Some("20131006".into()),
                            },
                            Property {
                                name: "SUMMARY".into(),
                                params: None,
                                value: Some("Birthday: Test Person".into()),
                            },
                        ]
                    }
                ]
            )
        )
    }
}
