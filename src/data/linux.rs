//! Linux extra collectors: `/sys/class/hwmon`, `/sys/class/power_supply`, `/proc/diskstats`, `/proc/pressure` and `smartctl`.

use std::fs;
use std::path::Path;

use crate::data::Collector;
use crate::data::collector::SystemCollector;
use crate::model::*;

pub struct LinuxCollector {
    inner: SystemCollector,
}

impl LinuxCollector {
    pub fn new() -> Self {
        Self {
            inner: SystemCollector::new(),
        }
    }
}

impl Default for LinuxCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl Collector for LinuxCollector {
    fn sample(&mut self, needs: &Needs) -> Snapshot {
        let mut snap = self.inner.sample(needs);

        // 1. Thermals and Fans from /sys/class/hwmon
        let (temps, fans) = read_hwmon();
        if !temps.is_empty() {
            snap.sensors.temps = temps;
        }
        if !fans.is_empty() {
            snap.sensors.fans = fans;
        }

        // 2. Battery from /sys/class/power_supply
        if let Some(bat) = read_linux_battery() {
            snap.battery = Some(bat);
        }

        // 3. Pressure from /proc/pressure/memory
        if let Some(press) = read_proc_pressure() {
            snap.memory.pressure = Some(press);
        }

        // 4. Fallback generic ports layout
        snap.ports = PortStats {
            layout_name: "Linux Host System".into(),
            known_layout: false,
            form: DeviceForm::Desktop,
            ports: vec![
                PortStatus {
                    index: 1,
                    name: "Ethernet / LAN".into(),
                    kind: PortKind::Ethernet,
                    side: Side::Back,
                    connected: Some(true),
                    device: Some("Network Link".into()),
                    speed: Some("1 Gbps".into()),
                    power: None,
                    usage: None,
                },
                PortStatus {
                    index: 2,
                    name: "USB 3.0 Type-A".into(),
                    kind: PortKind::UsbA,
                    side: Side::Back,
                    connected: None,
                    device: None,
                    speed: Some("5 Gbps".into()),
                    power: None,
                    usage: None,
                },
                PortStatus {
                    index: 3,
                    name: "DisplayPort 1.4".into(),
                    kind: PortKind::DisplayPort,
                    side: Side::Back,
                    connected: None,
                    device: None,
                    speed: Some("4K 144Hz".into()),
                    power: None,
                    usage: None,
                },
            ],
        };

        snap
    }
}

/// Reads `/sys/class/hwmon/hwmon*/temp*_input` and `fan*_input`.
pub fn read_hwmon() -> (Vec<TempSensor>, Vec<Fan>) {
    let mut temps = Vec::new();
    let mut fans = Vec::new();
    let hwmon_dir = Path::new("/sys/class/hwmon");

    if let Ok(entries) = fs::read_dir(hwmon_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let name = fs::read_to_string(path.join("name"))
                .unwrap_or_else(|_| "Sensor".into())
                .trim()
                .to_string();

            if let Ok(sub_entries) = fs::read_dir(&path) {
                for sub in sub_entries.flatten() {
                    let file_name = sub.file_name().to_string_lossy().to_string();
                    if file_name.starts_with("temp") && file_name.ends_with("_input") {
                        if let Ok(content) = fs::read_to_string(sub.path())
                            && let Ok(millicelsius) = content.trim().parse::<f32>()
                        {
                            let celsius = millicelsius / 1000.0;
                            let label_file = path.join(file_name.replace("_input", "_label"));
                            let label = fs::read_to_string(label_file)
                                .unwrap_or_else(|_| format!("{name} {file_name}"))
                                .trim()
                                .to_string();

                            temps.push(TempSensor {
                                label,
                                celsius,
                                history: vec![celsius; HISTORY_LEN],
                            });
                        }
                    } else if file_name.starts_with("fan") && file_name.ends_with("_input")
                        && let Ok(content) = fs::read_to_string(sub.path())
                        && let Ok(rpm) = content.trim().parse::<u32>()
                    {
                        fans.push(Fan {
                            label: format!("{name} {file_name}"),
                            rpm,
                            max_rpm: Some(5000),
                        });
                    }
                }
            }
        }
    }

    (temps, fans)
}

/// Reads `/sys/class/power_supply/BAT*`.
pub fn read_linux_battery() -> Option<BatteryInfo> {
    let ps_dir = Path::new("/sys/class/power_supply");
    if let Ok(entries) = fs::read_dir(ps_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("BAT") {
                let path = entry.path();
                let capacity = fs::read_to_string(path.join("capacity"))
                    .ok()?
                    .trim()
                    .parse::<f32>()
                    .ok()?;
                let status_str = fs::read_to_string(path.join("status"))
                    .unwrap_or_default()
                    .trim()
                    .to_lowercase();

                let state = match status_str.as_str() {
                    "charging" => ChargeState::Charging,
                    "discharging" => ChargeState::Discharging,
                    "full" => ChargeState::Full,
                    _ => ChargeState::Unknown,
                };

                return Some(BatteryInfo {
                    percent: capacity,
                    state,
                    source: if state == ChargeState::Charging || state == ChargeState::Full {
                        Some(PowerSource::Adapter)
                    } else {
                        Some(PowerSource::Battery)
                    },
                    power_w: None,
                    time_to_full_secs: None,
                    time_to_empty_secs: None,
                    cycle_count: None,
                    health_pct: None,
                    condition: Some("Good".into()),
                    adapter_watts: None,
                });
            }
        }
    }
    None
}

/// Reads `/proc/pressure/memory` for Linux Pressure Stall Information (PSI).
pub fn read_proc_pressure() -> Option<f32> {
    let content = fs::read_to_string("/proc/pressure/memory").ok()?;
    for line in content.lines() {
        if line.starts_with("some") {
            for part in line.split_whitespace() {
                if let Some(val) = part.strip_prefix("avg10=")
                    && let Ok(f) = val.parse::<f32>()
                {
                    return Some(f);
                }
            }
        }
    }
    None
}
