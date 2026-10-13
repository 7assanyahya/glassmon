//! The immutable data model.
//!
//! Collectors (`crate::data`) produce a [`Snapshot`] once per tick; the UI
//! (`crate::ui`) only ever reads it. Nothing in here talks to the operating
//! system, which is what lets contributors add a platform or a page without
//! touching the other side.
//!
//! Conventions:
//! - Anything a platform might not provide is an `Option` (rendered as `n/a`).
//! - Rates are bytes per second as `f64`; sizes are bytes as `u64`.
//! - Percentages are `0.0..=100.0`.
//! - Histories are oldest-first and at most [`HISTORY_LEN`] long.

/// Number of samples kept for every graph.
pub const HISTORY_LEN: usize = 60;

/// Everything the UI needs to draw one frame.
#[derive(Debug, Clone, Default)]
pub struct Snapshot {
    /// Monotonic sample counter, useful for animations and tests.
    pub seq: u64,
    pub host: HostInfo,
    pub cpu: CpuStats,
    pub memory: MemoryStats,
    pub network: NetworkStats,
    pub processes: Vec<ProcessInfo>,
    pub disks: DiskStats,
    pub sensors: SensorStats,
    pub battery: Option<BatteryInfo>,
    pub hardware: HardwareInfo,
    pub ports: PortStats,
}

/// Which parts of the snapshot the visible page needs. Collectors use this to
/// skip expensive work (process scans, sensor reads, helper commands).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Needs {
    pub processes: bool,
    /// Per-process extras that cost one file read per process (threads on Linux).
    pub process_details: bool,
    pub sensors: bool,
    pub disks: bool,
    /// Disk extras: IOPS/latency helpers, SMART, storage devices.
    pub disk_details: bool,
    pub hardware: bool,
    pub battery: bool,
}

impl Needs {
    /// True if `self` asks for something `other` did not.
    pub fn exceeds(&self, other: &Needs) -> bool {
        (self.processes && !other.processes)
            || (self.process_details && !other.process_details)
            || (self.sensors && !other.sensors)
            || (self.disks && !other.disks)
            || (self.disk_details && !other.disk_details)
            || (self.hardware && !other.hardware)
            || (self.battery && !other.battery)
    }

    /// Everything on (used by the demo collector and tests).
    pub fn all() -> Self {
        Self {
            processes: true,
            process_details: true,
            sensors: true,
            disks: true,
            disk_details: true,
            hardware: true,
            battery: true,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct HostInfo {
    pub hostname: Option<String>,
    /// e.g. "macOS 15.6" or "Ubuntu 24.04".
    pub os: Option<String>,
    pub kernel: Option<String>,
    pub uptime_secs: Option<u64>,
    pub load_avg: Option<[f64; 3]>,
    pub process_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CoreKind {
    Performance,
    Efficiency,
    #[default]
    Generic,
}

#[derive(Debug, Clone, Default)]
pub struct CoreUsage {
    /// "P0", "E3" or "C7".
    pub label: String,
    pub kind: CoreKind,
    pub usage: f32,
}

#[derive(Debug, Clone, Default)]
pub struct CpuStats {
    pub brand: Option<String>,
    pub usage: f32,
    pub cores: Vec<CoreUsage>,
    pub history: Vec<f32>,
    pub frequency_mhz: Option<u64>,
}

#[derive(Debug, Clone, Default)]
pub struct MemoryStats {
    pub total: u64,
    pub used: u64,
    pub available: u64,
    pub swap_total: u64,
    pub swap_used: u64,
    /// Memory pressure, 0..=100. `None` when the platform cannot tell.
    pub pressure: Option<f32>,
    pub history: Vec<f32>,
}

impl MemoryStats {
    pub fn used_pct(&self) -> f64 {
        pct(self.used, self.total)
    }
    pub fn swap_pct(&self) -> f64 {
        pct(self.swap_used, self.swap_total)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum IfaceKind {
    Wifi,
    Ethernet,
    Loopback,
    Virtual,
    #[default]
    Other,
}

impl IfaceKind {
    pub fn label(self) -> &'static str {
        match self {
            IfaceKind::Wifi => "Wi-Fi",
            IfaceKind::Ethernet => "Ethernet",
            IfaceKind::Loopback => "Loopback",
            IfaceKind::Virtual => "Virtual",
            IfaceKind::Other => "Other",
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct InterfaceStats {
    pub name: String,
    pub kind: IfaceKind,
    pub rx_rate: f64,
    pub tx_rate: f64,
    pub rx_total: u64,
    pub tx_total: u64,
    pub rx_history: Vec<f64>,
    pub tx_history: Vec<f64>,
    pub mac: Option<String>,
    pub addresses: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct NetworkStats {
    pub interfaces: Vec<InterfaceStats>,
    /// Totals over physical (non-loopback, non-virtual) interfaces.
    pub rx_rate: f64,
    pub tx_rate: f64,
    pub rx_history: Vec<f64>,
    pub tx_history: Vec<f64>,
}

#[derive(Debug, Clone, Default)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub user: Option<String>,
    /// Percent of one core (can exceed 100 on multi-threaded processes).
    pub cpu: f32,
    pub memory: u64,
    pub threads: Option<u32>,
    pub state: String,
    pub command: String,
    pub disk_read_rate: f64,
    pub disk_write_rate: f64,
}

#[derive(Debug, Clone, Default)]
pub struct Volume {
    /// Human name ("Macintosh HD", "nvme0n1p2").
    pub name: String,
    /// Device path or BSD name ("/dev/nvme0n1p2", "disk3s1").
    pub device: String,
    pub mount: String,
    pub fs: String,
    pub total: u64,
    pub available: u64,
    pub removable: bool,
    /// "SSD", "HDD" or "Unknown".
    pub kind: String,
}

impl Volume {
    pub fn used(&self) -> u64 {
        self.total.saturating_sub(self.available)
    }
    pub fn used_pct(&self) -> f64 {
        pct(self.used(), self.total)
    }
}

#[derive(Debug, Clone, Default)]
pub struct DiskIo {
    pub read_rate: f64,
    pub write_rate: f64,
    pub read_history: Vec<f64>,
    pub write_history: Vec<f64>,
    pub iops: Option<f64>,
    pub latency_ms: Option<f64>,
    pub queue_depth: Option<f64>,
    pub busy_pct: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StorageCategory {
    Apps,
    Developer,
    Media,
    Documents,
    System,
    Other,
    Used,
    Free,
}

impl StorageCategory {
    pub fn label(self) -> &'static str {
        match self {
            StorageCategory::Apps => "Apps",
            StorageCategory::Developer => "Developer",
            StorageCategory::Media => "Media",
            StorageCategory::Documents => "Documents",
            StorageCategory::System => "System",
            StorageCategory::Other => "Other",
            StorageCategory::Used => "Used",
            StorageCategory::Free => "Free",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ScanState {
    #[default]
    NotScanned,
    Scanning,
    Ready,
    Failed,
}

/// Breakdown of what uses space on one volume.
#[derive(Debug, Clone, Default)]
pub struct StorageBreakdown {
    pub mount: String,
    pub state: ScanState,
    /// Category segments (macOS boot volume) or `Used`/`Free` (elsewhere).
    pub segments: Vec<(StorageCategory, u64)>,
    /// Largest top-level directories (Linux and non-boot volumes).
    pub top_dirs: Vec<(String, u64)>,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HealthStatus {
    Verified,
    Warning,
    #[default]
    Unknown,
}

#[derive(Debug, Clone, Default)]
pub struct SmartInfo {
    pub device: String,
    pub model: Option<String>,
    pub status: HealthStatus,
    pub temperature_c: Option<f32>,
    pub power_on_hours: Option<u64>,
    pub data_written: Option<u64>,
    /// NVMe "percentage used" (wear), 0..=100+.
    pub wear_pct: Option<f32>,
    pub available_spare_pct: Option<f32>,
    pub unsafe_shutdowns: Option<u64>,
    pub media_errors: Option<u64>,
    /// Why details are missing ("needs sudo", "install smartmontools").
    pub note: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct DiskStats {
    pub volumes: Vec<Volume>,
    pub io: DiskIo,
    pub breakdowns: Vec<StorageBreakdown>,
    pub smart: Vec<SmartInfo>,
}

impl DiskStats {
    pub fn breakdown_for(&self, mount: &str) -> Option<&StorageBreakdown> {
        self.breakdowns.iter().find(|b| b.mount == mount)
    }
}

#[derive(Debug, Clone, Default)]
pub struct TempSensor {
    pub label: String,
    pub celsius: f32,
    pub history: Vec<f32>,
}

#[derive(Debug, Clone, Default)]
pub struct Fan {
    pub label: String,
    pub rpm: u32,
    pub max_rpm: Option<u32>,
}

#[derive(Debug, Clone, Default)]
pub struct SensorStats {
    pub temps: Vec<TempSensor>,
    pub fans: Vec<Fan>,
}

impl SensorStats {
    pub fn max_temp(&self) -> Option<f32> {
        self.temps.iter().map(|t| t.celsius).reduce(f32::max)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ChargeState {
    Charging,
    Discharging,
    Full,
    NotCharging,
    #[default]
    Unknown,
}

impl ChargeState {
    pub fn label(self) -> &'static str {
        match self {
            ChargeState::Charging => "Charging",
            ChargeState::Discharging => "Discharging",
            ChargeState::Full => "Fully charged",
            ChargeState::NotCharging => "Not charging",
            ChargeState::Unknown => "Unknown",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerSource {
    Adapter,
    Battery,
}

#[derive(Debug, Clone, Default)]
pub struct BatteryInfo {
    pub percent: f32,
    pub state: ChargeState,
    pub source: Option<PowerSource>,
    /// System draw in watts (battery discharge or charge rate).
    pub power_w: Option<f32>,
    pub time_to_full_secs: Option<u64>,
    pub time_to_empty_secs: Option<u64>,
    pub cycle_count: Option<u32>,
    /// Maximum capacity relative to design, percent.
    pub health_pct: Option<f32>,
    /// Vendor condition string ("Normal", "Service Recommended").
    pub condition: Option<String>,
    pub adapter_watts: Option<u32>,
}

#[derive(Debug, Clone, Default)]
pub struct WifiInfo {
    pub interface: String,
    pub ssid: Option<String>,
    pub link_mbps: Option<f64>,
    pub signal_dbm: Option<i32>,
    pub noise_dbm: Option<i32>,
    pub channel: Option<String>,
    pub phy_mode: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct BluetoothDevice {
    pub name: String,
    pub kind: Option<String>,
    pub connected: bool,
}

#[derive(Debug, Clone, Default)]
pub struct StorageDevice {
    pub name: String,
    pub interface: Option<String>,
    pub health: HealthStatus,
    pub size: Option<u64>,
    pub read_rate: Option<f64>,
    pub write_rate: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CoreCount {
    pub total: usize,
    pub performance: Option<usize>,
    pub efficiency: Option<usize>,
}

#[derive(Debug, Clone, Default)]
pub struct HardwareInfo {
    /// Marketing name ("MacBook Pro").
    pub model_name: Option<String>,
    /// Model identifier ("Mac15,6", "MacBookPro18,3"); keys the port layout table.
    pub model_id: Option<String>,
    pub chip: Option<String>,
    pub cores: Option<CoreCount>,
    pub gpu: Option<String>,
    pub gpu_cores: Option<u32>,
    pub memory_total: Option<u64>,
    pub display: Option<String>,
    pub wifi: Option<WifiInfo>,
    pub bluetooth_on: Option<bool>,
    pub bluetooth: Vec<BluetoothDevice>,
    pub storage: Vec<StorageDevice>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PortKind {
    Thunderbolt,
    UsbC,
    UsbA,
    MagSafe,
    Hdmi,
    SdCard,
    Headphone,
    Ethernet,
    DisplayPort,
    #[default]
    Other,
}

impl PortKind {
    /// Short label used under port badges.
    pub fn short(self) -> &'static str {
        match self {
            PortKind::Thunderbolt => "TB",
            PortKind::UsbC => "USB-C",
            PortKind::UsbA => "USB-A",
            PortKind::MagSafe => "MagSafe",
            PortKind::Hdmi => "HDMI",
            PortKind::SdCard => "SD",
            PortKind::Headphone => "Audio",
            PortKind::Ethernet => "LAN",
            PortKind::DisplayPort => "DP",
            PortKind::Other => "Port",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Side {
    Left,
    Right,
    Back,
    #[default]
    Unknown,
}

#[derive(Debug, Clone, Default)]
pub struct PortStatus {
    /// 1-based number shown on badges.
    pub index: u8,
    pub name: String,
    pub kind: PortKind,
    pub side: Side,
    /// `None` when the platform cannot tell whether something is plugged in.
    pub connected: Option<bool>,
    pub device: Option<String>,
    pub speed: Option<String>,
    pub power: Option<String>,
    /// Link utilisation 0..=1, when known.
    pub usage: Option<f32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DeviceForm {
    Laptop,
    Desktop,
    #[default]
    Unknown,
}

#[derive(Debug, Clone, Default)]
pub struct PortStats {
    /// Layout name, e.g. "MacBook Pro 14-inch (M3 Pro/Max, 2023)" or "Generic".
    pub layout_name: String,
    /// True when the ports come from the machine layout table.
    pub known_layout: bool,
    pub form: DeviceForm,
    pub ports: Vec<PortStatus>,
}

impl PortStats {
    pub fn in_use(&self) -> usize {
        self.ports
            .iter()
            .filter(|p| p.connected == Some(true))
            .count()
    }
}

/// `part / whole` as a percentage, 0 when `whole` is 0.
pub fn pct(part: u64, whole: u64) -> f64 {
    if whole == 0 {
        0.0
    } else {
        (part as f64 / whole as f64 * 100.0).clamp(0.0, 100.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pct_handles_zero_and_clamps() {
        assert_eq!(pct(1, 0), 0.0);
        assert_eq!(pct(50, 100), 50.0);
        assert_eq!(pct(200, 100), 100.0);
    }

    #[test]
    fn needs_exceeds() {
        let none = Needs::default();
        let procs = Needs {
            processes: true,
            ..Needs::default()
        };
        assert!(procs.exceeds(&none));
        assert!(!none.exceeds(&procs));
        assert!(!procs.exceeds(&procs));
    }

    #[test]
    fn volume_used_saturates() {
        let v = Volume {
            total: 10,
            available: 20,
            ..Volume::default()
        };
        assert_eq!(v.used(), 0);
    }
}
