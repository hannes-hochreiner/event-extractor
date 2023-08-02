use super::*;

#[test]
fn parse_test_1() {
    let text = r#"
        {
            "entries": [
                {"input": "/path/input", "output": "/path/output", "remove_files": true}
            ]
        }
    "#;

    assert_eq!(
        serde_json::from_str::<Config>(text).unwrap(),
        Config {
            entries: vec![Entry {
                input: "/path/input".into(),
                output: "/path/output".into(),
                remove_files: true
            }]
        }
    )
}
