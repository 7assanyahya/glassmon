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

#[cfg(target_os = "macos")]
#[test]
fn test_parse_intel_mac_system_profiler() {
    let raw = r#"{
      "SPHardwareDataType": [{
        "machine_name": "MacBook Pro",
        "machine_model": "MacBookPro15,1",
        "cpu_type": "8-Core Intel Core i9",
        "total_number_cores": 8,
        "number_processors": 16,
        "physical_memory": "16 GB"
      }],
      "SPDisplaysDataType": [
        {
          "_name": "kHW_IntelUHDGraphics630Item",
          "sppci_model": "Intel UHD Graphics 630",
          "_spdisplays_vram": "1536 MB"
        },
        {
          "_name": "kHW_AMDRadeonPro560XItem",
          "sppci_model": "Radeon Pro 560X",
          "spdisplays_vram": "4 GB"
        }
      ]
    }"#;
    let json: serde_json::Value = serde_json::from_str(raw).unwrap();
    let (hw, ports, _) = glassmon::data::macos::parse_profiler_json(&json);

    assert_eq!(hw.model_name.as_deref(), Some("MacBook Pro"));
    assert_eq!(hw.model_id.as_deref(), Some("MacBookPro15,1"));
    assert_eq!(hw.chip.as_deref(), Some("8-Core Intel Core i9"));

    let cores = hw.cores.unwrap();
    assert_eq!(cores.total, 16);
    assert_eq!(cores.performance, Some(8));

    assert!(hw.gpu.is_some());
    let gpu_str = hw.gpu.unwrap();
    assert!(gpu_str.contains("Intel UHD Graphics 630"));
    assert!(gpu_str.contains("Radeon Pro 560X"));

    assert!(ports.known_layout);
    assert_eq!(ports.ports.len(), 5);
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
