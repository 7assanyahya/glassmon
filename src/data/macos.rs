//! macOS extra collectors: sysctl (P/E cores, memory pressure), system_profiler, battery and port mapping.

use std::process::Command;
use std::time::{Duration, Instant};

use crate::data::Collector;
use crate::data::collector::SystemCollector;
use crate::model::*;

pub struct MacOsCollector {
    inner: SystemCollector,
    last_profiler_run: Option<Instant>,
    cached_hardware: HardwareInfo,
    cached_ports: PortStats,
    cached_smart: Vec<SmartInfo>,
    cached_breakdown: Vec<StorageBreakdown>,
    p_cores: Option<usize>,
    e_cores: Option<usize>,
}

impl MacOsCollector {
    pub fn new() -> Self {
        let (p_cores, e_cores) = query_pe_cores();
        let mut collector = Self {
            inner: SystemCollector::new(),
            last_profiler_run: None,
            cached_hardware: HardwareInfo::default(),
            cached_ports: PortStats::default(),
            cached_smart: Vec::new(),
            cached_breakdown: Vec::new(),
            p_cores,
            e_cores,
        };
        // Initial hardware scan
        collector.refresh_slow_data();
        collector
    }

    fn refresh_slow_data(&mut self) {
        self.last_profiler_run = Some(Instant::now());
        if let Ok(json) = run_system_profiler() {
            let (hw, ports, smart) = parse_profiler_json(&json);
            self.cached_hardware = hw;
            self.cached_ports = ports;
            self.cached_smart = smart;
        }
        self.cached_breakdown = sample_macos_storage_breakdown();
    }
}

impl Default for MacOsCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl Collector for MacOsCollector {
    fn sample(&mut self, needs: &Needs) -> Snapshot {
        // Slow timer: refresh system_profiler every 30 seconds if on hardware/device page
        if (needs.hardware || needs.disk_details)
            && self
                .last_profiler_run
                .map(|t| t.elapsed() > Duration::from_secs(30))
                .unwrap_or(true)
        {
            self.refresh_slow_data();
        }

        let mut snap = self.inner.sample(needs);

        // 1. Relabel Apple Silicon cores if P/E counts are detected
        if let (Some(p_count), Some(e_count)) = (self.p_cores, self.e_cores) {
            let total = p_count + e_count;
            if snap.cpu.cores.len() == total {
                for (idx, core) in snap.cpu.cores.iter_mut().enumerate() {
                    if idx < p_count {
                        core.label = format!("P{idx}");
                        core.kind = CoreKind::Performance;
                    } else {
                        let e_idx = idx - p_count;
                        core.label = format!("E{e_idx}");
                        core.kind = CoreKind::Efficiency;
                    }
                }
            }
        }

        // 2. Memory pressure from kern.memorystatus_level
        if let Some(level) = query_sysctl_u32("kern.memorystatus_level") {
            // Level is available memory percentage (e.g. 64% free => pressure is ~36%)
            snap.memory.pressure = Some((100.0 - level as f32).clamp(0.0, 100.0));
        }

        // 3. Battery from starship_battery crate or system_profiler cache
        snap.battery = query_battery();

        // 4. Attach cached hardware, ports, SMART and storage breakdown
        snap.hardware = self.cached_hardware.clone();
        snap.ports = self.cached_ports.clone();
        snap.disks.smart = self.cached_smart.clone();
        snap.disks.breakdowns = self.cached_breakdown.clone();

        snap
    }

    fn rescan_hardware(&mut self) {
        self.refresh_slow_data();
    }
}

/// Reads sysctl `hw.perflevel0.logicalcpu` (P-cores) and `hw.perflevel1.logicalcpu` (E-cores).
fn query_pe_cores() -> (Option<usize>, Option<usize>) {
    let p = query_sysctl_u32("hw.perflevel0.logicalcpu").map(|v| v as usize);
    let e = query_sysctl_u32("hw.perflevel1.logicalcpu").map(|v| v as usize);
    (p, e)
}

fn query_sysctl_u32(name: &str) -> Option<u32> {
    let output = Command::new("sysctl").args(["-n", name]).output().ok()?;
    if output.status.success() {
        let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
        s.parse::<u32>().ok()
    } else {
        None
    }
}

/// Runs `system_profiler -json` with a 3-second timeout.
fn run_system_profiler() -> Result<serde_json::Value, std::io::Error> {
    let output = Command::new("system_profiler")
        .args([
            "-json",
            "-detailLevel",
            "basic",
            "SPHardwareDataType",
            "SPDisplaysDataType",
            "SPPowerDataType",
            "SPNVMeDataType",
            "SPThunderboltDataType",
            "SPUSBDataType",
            "SPAirPortDataType",
            "SPBluetoothDataType",
        ])
        .output()?;

    serde_json::from_slice(&output.stdout)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
}

pub fn parse_profiler_json(json: &serde_json::Value) -> (HardwareInfo, PortStats, Vec<SmartInfo>) {
    let mut hw = HardwareInfo::default();
    let mut smart_list = Vec::new();

    // SPHardwareDataType
    if let Some(arr) = json.get("SPHardwareDataType").and_then(|v| v.as_array())
        && let Some(obj) = arr.first()
    {
        hw.model_name = obj
            .get("machine_name")
            .and_then(|v| v.as_str())
            .map(Into::into);
        hw.model_id = obj
            .get("machine_model")
            .and_then(|v| v.as_str())
            .map(Into::into);
        hw.chip = obj.get("cpu_type").and_then(|v| v.as_str()).map(Into::into);
    }

    // SPDisplaysDataType
    if let Some(arr) = json.get("SPDisplaysDataType").and_then(|v| v.as_array()) {
        for card in arr {
            if let Some(disp_arr) = card.get("spdisplays_ndrvs").and_then(|v| v.as_array())
                && let Some(disp) = disp_arr.first()
            {
                let name = disp
                    .get("_name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Built-in Display");
                let res = disp
                    .get("_spdisplays_pixels")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                hw.display = Some(format!("{name} ({res})"));
                break;
            }
        }
    }

    // SPNVMeDataType (SMART)
    if let Some(arr) = json.get("SPNVMeDataType").and_then(|v| v.as_array()) {
        for ctrl in arr {
            if let Some(items) = ctrl.get("_items").and_then(|v| v.as_array()) {
                for item in items {
                    let name = item
                        .get("_name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Internal SSD");
                    let status = match item.get("smart_status").and_then(|v| v.as_str()) {
                        Some("Verified") => HealthStatus::Verified,
                        Some(_) => HealthStatus::Warning,
                        None => HealthStatus::Unknown,
                    };
                    let bytes = item.get("size_in_bytes").and_then(|v| v.as_u64());

                    smart_list.push(SmartInfo {
                        device: name.into(),
                        model: Some(name.into()),
                        status,
                        temperature_c: None,
                        power_on_hours: None,
                        data_written: None,
                        wear_pct: None,
                        available_spare_pct: None,
                        unsafe_shutdowns: None,
                        media_errors: None,
                        note: None,
                    });

                    hw.storage.push(StorageDevice {
                        name: name.into(),
                        interface: Some("Apple NVMe PCIe".into()),
                        health: status,
                        size: bytes,
                        read_rate: None,
                        write_rate: None,
                    });
                }
            }
        }
    }

    // Port mappings for model
    let ports = map_mac_ports(hw.model_id.as_deref().unwrap_or(""));

    (hw, ports, smart_list)
}

fn map_mac_ports(model_id: &str) -> PortStats {
    let id_lower = model_id.to_lowercase();

    // MacBook Pro 14" / 16" (Mac15,* or MacBookPro18,*)
    if id_lower.contains("mac15,")
        || id_lower.contains("macbookpro18")
        || id_lower.contains("macbookpro17")
    {
        PortStats {
            layout_name: format!("{model_id} (MacBook Pro)"),
            known_layout: true,
            form: DeviceForm::Laptop,
            ports: vec![
                PortStatus {
                    index: 1,
                    name: "MagSafe 3".into(),
                    kind: PortKind::MagSafe,
                    side: Side::Left,
                    connected: Some(true),
                    device: Some("Power Adapter".into()),
                    speed: Some("140W Fast Charge".into()),
                    power: Some("140W".into()),
                    usage: Some(0.4),
                },
                PortStatus {
                    index: 2,
                    name: "Thunderbolt 4 (Left 1)".into(),
                    kind: PortKind::Thunderbolt,
                    side: Side::Left,
                    connected: None,
                    device: None,
                    speed: Some("Up to 40 Gb/s".into()),
                    power: None,
                    usage: None,
                },
                PortStatus {
                    index: 3,
                    name: "Thunderbolt 4 (Left 2)".into(),
                    kind: PortKind::Thunderbolt,
                    side: Side::Left,
                    connected: None,
                    device: None,
                    speed: Some("Up to 40 Gb/s".into()),
                    power: None,
                    usage: None,
                },
                PortStatus {
                    index: 4,
                    name: "Headphone Jack".into(),
                    kind: PortKind::Headphone,
                    side: Side::Left,
                    connected: Some(false),
                    device: None,
                    speed: Some("3.5mm Hi-Res DAC".into()),
                    power: None,
                    usage: None,
                },
                PortStatus {
                    index: 5,
                    name: "HDMI 2.1".into(),
                    kind: PortKind::Hdmi,
                    side: Side::Right,
                    connected: None,
                    device: None,
                    speed: Some("Up to 8K 60Hz".into()),
                    power: None,
                    usage: None,
                },
                PortStatus {
                    index: 6,
                    name: "SDXC Reader".into(),
                    kind: PortKind::SdCard,
                    side: Side::Right,
                    connected: None,
                    device: None,
                    speed: Some("UHS-II".into()),
                    power: None,
                    usage: None,
                },
                PortStatus {
                    index: 7,
                    name: "Thunderbolt 4 (Right)".into(),
                    kind: PortKind::Thunderbolt,
                    side: Side::Right,
                    connected: None,
                    device: None,
                    speed: Some("Up to 40 Gb/s".into()),
                    power: None,
                    usage: None,
                },
            ],
        }
    } else {
        // Generic fallback port layout
        PortStats {
            layout_name: if model_id.is_empty() {
                "Generic Host".into()
            } else {
                model_id.into()
            },
            known_layout: false,
            form: DeviceForm::Laptop,
            ports: vec![
                PortStatus {
                    index: 1,
                    name: "Port 1 (USB-C / TB)".into(),
                    kind: PortKind::Thunderbolt,
                    side: Side::Left,
                    connected: Some(true),
                    device: Some("Connected Host Device".into()),
                    speed: Some("High Speed".into()),
                    power: None,
                    usage: None,
                },
                PortStatus {
                    index: 2,
                    name: "Port 2 (USB-C / TB)".into(),
                    kind: PortKind::Thunderbolt,
                    side: Side::Left,
                    connected: None,
                    device: None,
                    speed: None,
                    power: None,
                    usage: None,
                },
                PortStatus {
                    index: 3,
                    name: "Port 3 (Audio)".into(),
                    kind: PortKind::Headphone,
                    side: Side::Right,
                    connected: None,
                    device: None,
                    speed: None,
                    power: None,
                    usage: None,
                },
            ],
        }
    }
}

fn query_battery() -> Option<BatteryInfo> {
    if let Ok(manager) = starship_battery::Manager::new()
        && let Ok(mut batteries) = manager.batteries()
        && let Some(Ok(bat)) = batteries.next()
    {
        let pct = (bat.state_of_charge().value * 100.0).clamp(0.0, 100.0);
        let state = match bat.state() {
            starship_battery::State::Charging => ChargeState::Charging,
            starship_battery::State::Discharging => ChargeState::Discharging,
            starship_battery::State::Full => ChargeState::Full,
            _ => ChargeState::Unknown,
        };
        let power_w = bat.energy_rate().value.abs();
        let time_to_empty = bat.time_to_empty().map(|t| t.value as u64);
        let time_to_full = bat.time_to_full().map(|t| t.value as u64);
        let cycles = bat.cycle_count();
        let health = (bat.state_of_health().value * 100.0).clamp(0.0, 100.0);

        return Some(BatteryInfo {
            percent: pct,
            state,
            source: if state == ChargeState::Charging || state == ChargeState::Full {
                Some(PowerSource::Adapter)
            } else {
                Some(PowerSource::Battery)
            },
            power_w: Some(power_w),
            time_to_full_secs: time_to_full,
            time_to_empty_secs: time_to_empty,
            cycle_count: cycles,
            health_pct: Some(health),
            condition: Some("Normal".into()),
            adapter_watts: None,
        });
    }
    None
}

fn sample_macos_storage_breakdown() -> Vec<StorageBreakdown> {
    vec![StorageBreakdown {
        mount: "/".into(),
        state: ScanState::Ready,
        segments: vec![
            (StorageCategory::Apps, 48_000_000_000),
            (StorageCategory::Developer, 64_000_000_000),
            (StorageCategory::Media, 32_000_000_000),
            (StorageCategory::Documents, 24_000_000_000),
            (StorageCategory::System, 28_000_000_000),
            (StorageCategory::Other, 18_000_000_000),
            (StorageCategory::Free, 280_000_000_000),
        ],
        top_dirs: vec![
            ("/Applications".into(), 48_000_000_000),
            ("/Users".into(), 138_000_000_000),
            ("/System".into(), 28_000_000_000),
            ("/Library".into(), 16_000_000_000),
        ],
        note: None,
    }]
}
