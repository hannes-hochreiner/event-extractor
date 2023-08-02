#[cfg(test)]
mod tests;

use ical::{
    parser::ical::component::{IcalCalendar, IcalEvent},
    property::Property,
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SerializationError {
    #[error("serialization of property \"{}\" not implemented", .0)]
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
