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
