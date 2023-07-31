use anyhow::anyhow;
use chrono::{Datelike, Utc};
use clap::Parser;
use event_extractor::{self, convert, ser::calendar_to_string};
use ical::{parser::ical::component::IcalCalendar, property::Property};
use std::fs::{read_dir, remove_file, File};
use std::io::{BufReader, BufWriter, Write};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input directory
    #[arg(short, long)]
    input: String,

    /// Output directory
    #[arg(short, long)]
    output: String,
}

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let args = Args::parse();

    // remove existing files
    log::info!("removing exiting files");
    for entry in read_dir(&args.output)? {
        let path = entry?.path();

        if path.is_file() && path.ends_with(".ics") {
            remove_file(path)?
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
    for entry in read_dir(&args.input)? {
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
                match vcard {
                    Ok(contact) => {
                        for event in convert(&contact, &years)? {
                            let uid = event
                                .properties
                                .iter()
                                .find(|&elem| elem.name == "UID")
                                .ok_or(anyhow!("could not find UID"))?
                                .value
                                .as_ref()
                                .ok_or(anyhow!("could not get UID value"))?
                                .clone();
                            let mut filename = PathBuf::from(&args.output);
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
                    Err(e) => log::error!(
                        "could not parse \"{}\": {:?}",
                        path.to_str()
                            .ok_or(anyhow!("could not convert path to str"))?,
                        e
                    ),
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use ical::parser::{vcard::component::VcardContact, ParserError};

    use super::*;

    #[test]
    fn parse_test() {
        let buf = BufReader::new(File::open("testfiles/test.vcf").unwrap());
        let reader =
            ical::VcardParser::new(buf).collect::<Vec<Result<VcardContact, ParserError>>>();

        assert_eq!(reader.len(), 1);
        assert_eq!(
            format!("{:?}", reader[0].as_ref().unwrap()),
            "VcardContact { properties: [Property { name: \"VERSION\", params: None, value: Some(\"4.0\") }, Property { name: \"UID\", params: None, value: Some(\"931bfb48-9437-483e-a2b8-7f8d56fdfba4\") }, Property { name: \"ANNIVERSARY\", params: Some([(\"VALUE\", [\"DATE\"])]), value: Some(\"20121005\") }, Property { name: \"BDAY\", params: Some([(\"VALUE\", [\"DATE\"])]), value: Some(\"19860328\") }, Property { name: \"FN\", params: None, value: Some(\"Testname Testsurname\") }, Property { name: \"N\", params: None, value: Some(\"Testsurname;Testname;;;\") }] }"
        );
    }

    #[test]
    fn parse_fold_test() {
        let buf = BufReader::new(File::open("testfiles/test_fold.vcf").unwrap());
        let reader =
            ical::VcardParser::new(buf).collect::<Vec<Result<VcardContact, ParserError>>>();

        assert_eq!(reader.len(), 1);
        assert_eq!(
            String::from_utf8(
                quoted_printable::decode(
                    reader[0]
                        .as_ref()
                        .unwrap()
                        .properties
                        .iter()
                        .find(|&elem| elem.name == "FN")
                        .unwrap()
                        .value
                        .as_ref()
                        .unwrap(),
                    quoted_printable::ParseMode::Robust,
                )
                .unwrap(),
            )
            .unwrap(),
            String::from("Testname Testsurname")
        );
    }

    #[test]
    fn parse_quote_test() {
        let buf = BufReader::new(File::open("testfiles/test_quoted.vcf").unwrap());
        let reader =
            ical::VcardParser::new(buf).collect::<Vec<Result<VcardContact, ParserError>>>();

        assert_eq!(reader.len(), 1);
        assert_eq!(
            String::from_utf8(
                quoted_printable::decode(
                    reader[0]
                        .as_ref()
                        .unwrap()
                        .properties
                        .iter()
                        .find(|&elem| elem.name == "FN")
                        .unwrap()
                        .value
                        .as_ref()
                        .unwrap(),
                    quoted_printable::ParseMode::Robust,
                )
                .unwrap(),
            )
            .unwrap(),
            String::from("☀️")
        );
    }
}
