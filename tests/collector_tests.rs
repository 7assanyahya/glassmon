//! Integration tests for data parsers and fixtures.

use glassmon::model::HealthStatus;
use std::fs;

#[cfg(target_os = "macos")]
#[test]
fn test_parse_system_profiler_fixture() {
    let fixture_str = fs::read_to_string("tests/fixtures/system_profiler.json")
        .expect("Failed to read system_profiler.json fixture");
    let json: serde_json::Value = serde_json::from_str(&fixture_str).unwrap();

    let (hw, ports, smart) = glassmon::data::macos::parse_profiler_json(&json);

    assert_eq!(hw.model_name.as_deref(), Some("MacBook Pro"));
    assert_eq!(hw.model_id.as_deref(), Some("Mac15,6"));
    assert_eq!(hw.chip.as_deref(), Some("Apple M3 Pro"));

    assert!(hw.display.is_some());
    assert!(hw.display.unwrap().contains("3024 x 1964"));

    assert_eq!(smart.len(), 1);
    assert_eq!(smart[0].status, HealthStatus::Verified);
    assert_eq!(smart[0].device, "APPLE SSD AP0512R");

    assert!(ports.known_layout);
    assert_eq!(ports.ports.len(), 7);
}

#[test]
fn test_linux_psi_parser_logic() {
    // Test the parsing string logic for Linux pressure
    let sample_psi = "some avg10=2.45 avg60=1.12 avg300=0.50 total=123456\nfull avg10=0.00 avg60=0.00 avg300=0.00 total=0\n";
    let mut val = None;
    for line in sample_psi.lines() {
        if line.starts_with("some") {
            for part in line.split_whitespace() {
                if let Some(v) = part.strip_prefix("avg10=") {
                    val = v.parse::<f32>().ok();
                }
            }
        }
    }
    assert_eq!(val, Some(2.45));
}
