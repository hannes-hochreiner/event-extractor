use ical::{
    parser::ical::component::{IcalCalendar, IcalEvent},
    property::Property,
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SerializationError {
    #[error("serialization of property \"{}\" not implemented", 0)]
    SerializationNotImplemented(String),
}

pub fn calendar_to_string(calendar: &IcalCalendar) -> Result<String, SerializationError> {
    if calendar.alarms.len() > 0 {
        return Err(SerializationError::SerializationNotImplemented(
            "calendar.alarms".into(),
        ));
    }

    if calendar.free_busys.len() > 0 {
        return Err(SerializationError::SerializationNotImplemented(
            "calendar.free_busys".into(),
        ));
    }

    if calendar.journals.len() > 0 {
        return Err(SerializationError::SerializationNotImplemented(
            "calendar.journals".into(),
        ));
    }

    if calendar.timezones.len() > 0 {
        return Err(SerializationError::SerializationNotImplemented(
            "calendar.timezones".into(),
        ));
    }

    if calendar.todos.len() > 0 {
        return Err(SerializationError::SerializationNotImplemented(
            "calendar.todos".into(),
        ));
    }

    Ok(format!(
        "BEGIN:VCALENDAR\r\n{}{}END:VCALENDAR\r\n",
        calendar
            .properties
            .iter()
            .map(|property| property_to_string(property))
            .collect::<Vec<String>>()
            .join(""),
        calendar
            .events
            .iter()
            .map(|event| event_to_string(event))
            .collect::<Result<Vec<String>, SerializationError>>()?
            .join("")
    ))
}

pub fn event_to_string(event: &IcalEvent) -> Result<String, SerializationError> {
    if event.alarms.len() > 0 {
        return Err(SerializationError::SerializationNotImplemented(
            "event.alarms".into(),
        ));
    }

    Ok(format!(
        "BEGIN:VEVENT\r\n{}END:VEVENT\r\n",
        event
            .properties
            .iter()
            .map(|property| property_to_string(&property))
            .collect::<Vec<String>>()
            .join("")
    ))
}

pub fn property_to_string(property: &Property) -> String {
    let mut out = property.name.to_owned();

    if let Some(params) = &property.params {
        for param in params {
            out += &format!(";{}={}", param.0, param.1.join(","));
        }
    }

    if let Some(value) = &property.value {
        out += &format!(":{}", value);
    }

    out += "\r\n";
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calendar_to_string_1() {
        assert_eq!(
            calendar_to_string(&IcalCalendar {
                properties: vec![
                    Property {
                        name: "VERSION".into(),
                        value: Some("2.0".into()),
                        ..Default::default()
                    },
                    Property {
                        name: "PROP2".into(),
                        value: Some("value2".into()),
                        ..Default::default()
                    }
                ],
                events: vec![
                    IcalEvent {
                        ..Default::default()
                    },
                    IcalEvent {
                        ..Default::default()
                    }
                ],
                ..Default::default()
            })
            .unwrap(),
            "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPROP2:value2\r\nBEGIN:VEVENT\r\nEND:VEVENT\r\nBEGIN:VEVENT\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n"
        );
    }

    #[test]
    fn event_to_string_1() {
        assert_eq!(
            event_to_string(&IcalEvent {
                properties: vec![
                    Property {
                        name: "UID".into(),
                        value: Some("test_uid".into()),
                        ..Default::default()
                    },
                    Property {
                        name: "PROP2".into(),
                        value: Some("value2".into()),
                        ..Default::default()
                    }
                ],
                alarms: Vec::new()
            })
            .unwrap(),
            "BEGIN:VEVENT\r\nUID:test_uid\r\nPROP2:value2\r\nEND:VEVENT\r\n"
        )
    }

    #[test]
    fn property_to_string_1() {
        assert_eq!(
            property_to_string(&Property {
                name: "test prop".into(),
                params: Some(vec![
                    ("param1".into(), vec!["param1_1".into(), "param1_2".into()]),
                    ("param2".into(), vec!["param2_1".into()])
                ]),
                value: Some("value1".into())
            }),
            "test prop;param1=param1_1,param1_2;param2=param2_1:value1\r\n"
        )
    }
}
