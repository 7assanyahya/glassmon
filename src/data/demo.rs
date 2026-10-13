//! Deterministic animated demo collector for screenshots, testing and dev.

use crate::data::Collector;
use crate::model::*;

pub struct DemoCollector {
    seq: u64,
}

impl DemoCollector {
    pub fn new() -> Self {
        Self { seq: 0 }
    }
}

impl Default for DemoCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl Collector for DemoCollector {
    fn sample(&mut self, _needs: &Needs) -> Snapshot {
        self.seq += 1;
        let t = self.seq as f64 * 0.15;

        // Animate 60-sample histories
        let mut cpu_history = Vec::with_capacity(HISTORY_LEN);
        let mut mem_history = Vec::with_capacity(HISTORY_LEN);
        let mut rx_history = Vec::with_capacity(HISTORY_LEN);
        let mut tx_history = Vec::with_capacity(HISTORY_LEN);
        let mut disk_read_history = Vec::with_capacity(HISTORY_LEN);
        let mut disk_write_history = Vec::with_capacity(HISTORY_LEN);
        let mut temp_history = Vec::with_capacity(HISTORY_LEN);

        for i in 0..HISTORY_LEN {
            let offset = (i as f64 - HISTORY_LEN as f64) * 0.12 + t;
            let c =
                (28.0 + 18.0 * offset.sin() + 8.0 * (offset * 2.3).cos()).clamp(4.0, 96.0) as f32;
            let m = (52.0 + 4.0 * (offset * 0.5).sin()).clamp(10.0, 95.0) as f32;
            let rx = 12_500_000.0 * (1.0 + (offset * 1.1).sin()).max(0.05);
            let tx = 2_400_000.0 * (1.0 + (offset * 1.7).cos()).max(0.02);
            let dr = 45_000_000.0 * (1.0 + (offset * 0.8).sin()).max(0.01);
            let dw = 18_000_000.0 * (1.0 + (offset * 1.4).cos()).max(0.01);
            let tmp = (46.0 + 8.0 * (offset * 0.9).sin()) as f32;

            cpu_history.push(c);
            mem_history.push(m);
            rx_history.push(rx);
            tx_history.push(tx);
            disk_read_history.push(dr);
            disk_write_history.push(dw);
            temp_history.push(tmp);
        }

        let curr_cpu = *cpu_history.last().unwrap_or(&32.0);
        let curr_rx = *rx_history.last().unwrap_or(&4_500_000.0);
        let curr_tx = *tx_history.last().unwrap_or(&800_000.0);
        let curr_dr = *disk_read_history.last().unwrap_or(&12_000_000.0);
        let curr_dw = *disk_write_history.last().unwrap_or(&5_000_000.0);

        // 12 Apple Silicon cores: 6 P-cores and 6 E-cores
        let mut cores = Vec::new();
        for i in 0..6 {
            let usage = ((curr_cpu * 1.1) + ((t + i as f64).sin() * 15.0) as f32).clamp(2.0, 100.0);
            cores.push(CoreUsage {
                label: format!("P{i}"),
                kind: CoreKind::Performance,
                usage,
            });
        }
        for i in 0..6 {
            let usage =
                ((curr_cpu * 0.7) + ((t + (i + 6) as f64).cos() * 10.0) as f32).clamp(1.0, 95.0);
            cores.push(CoreUsage {
                label: format!("E{i}"),
                kind: CoreKind::Efficiency,
                usage,
            });
        }

        let processes = vec![
            ProcessInfo {
                pid: 4812,
                name: "glassmon".into(),
                user: Some("hassan".into()),
                cpu: 1.8,
                memory: 38 * 1024 * 1024,
                threads: Some(1),
                state: "R".into(),
                command: "glassmon --demo".into(),
                disk_read_rate: 120_000.0,
                disk_write_rate: 45_000.0,
            },
            ProcessInfo {
                pid: 1984,
                name: "rust-analyzer".into(),
                user: Some("hassan".into()),
                cpu: 14.5,
                memory: 1420 * 1024 * 1024,
                threads: Some(16),
                state: "S".into(),
                command: "rust-analyzer --proc-macro-srv".into(),
                disk_read_rate: 2_400_000.0,
                disk_write_rate: 850_000.0,
            },
            ProcessInfo {
                pid: 240,
                name: "WindowServer".into(),
                user: Some("_windowserver".into()),
                cpu: 11.2,
                memory: 680 * 1024 * 1024,
                threads: Some(12),
                state: "S".into(),
                command: "/System/Library/CoreServices/WindowServer -display".into(),
                disk_read_rate: 10_000.0,
                disk_write_rate: 0.0,
            },
            ProcessInfo {
                pid: 3410,
                name: "cargo".into(),
                user: Some("hassan".into()),
                cpu: 48.0,
                memory: 890 * 1024 * 1024,
                threads: Some(8),
                state: "R".into(),
                command: "cargo check --all-targets".into(),
                disk_read_rate: 15_200_000.0,
                disk_write_rate: 12_800_000.0,
            },
            ProcessInfo {
                pid: 1422,
                name: "Ghostty".into(),
                user: Some("hassan".into()),
                cpu: 4.2,
                memory: 195 * 1024 * 1024,
                threads: Some(6),
                state: "S".into(),
                command: "/Applications/Ghostty.app/Contents/MacOS/ghostty".into(),
                disk_read_rate: 40_000.0,
                disk_write_rate: 8_000.0,
            },
            ProcessInfo {
                pid: 928,
                name: "Safari".into(),
                user: Some("hassan".into()),
                cpu: 3.5,
                memory: 850 * 1024 * 1024,
                threads: Some(24),
                state: "S".into(),
                command: "/Applications/Safari.app/Contents/MacOS/Safari".into(),
                disk_read_rate: 340_000.0,
                disk_write_rate: 60_000.0,
            },
            ProcessInfo {
                pid: 310,
                name: "docker".into(),
                user: Some("hassan".into()),
                cpu: 8.4,
                memory: 2400 * 1024 * 1024,
                threads: Some(32),
                state: "S".into(),
                command: "com.docker.backend --hypervisor".into(),
                disk_read_rate: 1_200_000.0,
                disk_write_rate: 980_000.0,
            },
            ProcessInfo {
                pid: 88,
                name: "launchd".into(),
                user: Some("root".into()),
                cpu: 0.2,
                memory: 45 * 1024 * 1024,
                threads: Some(3),
                state: "S".into(),
                command: "/sbin/launchd".into(),
                disk_read_rate: 2_000.0,
                disk_write_rate: 1_000.0,
            },
            ProcessInfo {
                pid: 1205,
                name: "Slack".into(),
                user: Some("hassan".into()),
                cpu: 2.1,
                memory: 920 * 1024 * 1024,
                threads: Some(18),
                state: "S".into(),
                command: "/Applications/Slack.app/Contents/MacOS/Slack".into(),
                disk_read_rate: 50_000.0,
                disk_write_rate: 20_000.0,
            },
            ProcessInfo {
                pid: 512,
                name: "kernel_task".into(),
                user: Some("root".into()),
                cpu: 5.6,
                memory: 410 * 1024 * 1024,
                threads: Some(184),
                state: "R".into(),
                command: "kernel_task".into(),
                disk_read_rate: 5_400_000.0,
                disk_write_rate: 3_200_000.0,
            },
        ];

        let volumes = vec![
            Volume {
                name: "Macintosh HD".into(),
                device: "/dev/disk3s1".into(),
                mount: "/".into(),
                fs: "apfs".into(),
                total: 494_384_000_000,
                available: 218_500_000_000,
                removable: false,
                kind: "SSD".into(),
            },
            Volume {
                name: "Workspace SSD".into(),
                device: "/dev/disk4s1".into(),
                mount: "/Volumes/Workspace".into(),
                fs: "apfs".into(),
                total: 1_000_000_000_000,
                available: 640_000_000_000,
                removable: true,
                kind: "SSD".into(),
            },
        ];

        let breakdown = StorageBreakdown {
            mount: "/".into(),
            state: ScanState::Ready,
            segments: vec![
                (StorageCategory::Apps, 64_000_000_000),
                (StorageCategory::Developer, 78_500_000_000),
                (StorageCategory::Media, 42_000_000_000),
                (StorageCategory::Documents, 31_200_000_000),
                (StorageCategory::System, 36_800_000_000),
                (StorageCategory::Other, 23_384_000_000),
                (StorageCategory::Free, 218_500_000_000),
            ],
            top_dirs: vec![
                ("/Users/hassan/src".into(), 68_000_000_000),
                ("/Applications".into(), 64_000_000_000),
                ("/System".into(), 36_800_000_000),
                ("/Library".into(), 18_400_000_000),
            ],
            note: None,
        };

        let smart = vec![SmartInfo {
            device: "disk0 (APPLE SSD AP0512R)".into(),
            model: Some("APPLE SSD AP0512R".into()),
            status: HealthStatus::Verified,
            temperature_c: Some(34.0),
            power_on_hours: Some(1840),
            data_written: Some(42_500_000_000_000),
            wear_pct: Some(3.0),
            available_spare_pct: Some(100.0),
            unsafe_shutdowns: Some(4),
            media_errors: Some(0),
            note: None,
        }];

        let interfaces = vec![
            InterfaceStats {
                name: "en0".into(),
                kind: IfaceKind::Wifi,
                rx_rate: curr_rx,
                tx_rate: curr_tx,
                rx_total: 48_500_000_000,
                tx_total: 12_400_000_000,
                rx_history: rx_history.clone(),
                tx_history: tx_history.clone(),
                mac: Some("3c:06:30:1a:8b:20".into()),
                addresses: vec!["192.168.1.142".into(), "fe80::1044:ab71".into()],
            },
            InterfaceStats {
                name: "en1".into(),
                kind: IfaceKind::Ethernet,
                rx_rate: 0.0,
                tx_rate: 0.0,
                rx_total: 0,
                tx_total: 0,
                rx_history: vec![0.0; HISTORY_LEN],
                tx_history: vec![0.0; HISTORY_LEN],
                mac: Some("3c:06:30:1a:8b:21".into()),
                addresses: vec![],
            },
            InterfaceStats {
                name: "lo0".into(),
                kind: IfaceKind::Loopback,
                rx_rate: 450_000.0,
                tx_rate: 450_000.0,
                rx_total: 2_100_000_000,
                tx_total: 2_100_000_000,
                rx_history: vec![450_000.0; HISTORY_LEN],
                tx_history: vec![450_000.0; HISTORY_LEN],
                mac: None,
                addresses: vec!["127.0.0.1".into(), "::1".into()],
            },
        ];

        let ports = vec![
            PortStatus {
                index: 1,
                name: "MagSafe 3".into(),
                kind: PortKind::MagSafe,
                side: Side::Left,
                connected: Some(true),
                device: Some("140W USB-C Power Adapter".into()),
                speed: Some("140W".into()),
                power: Some("140W PD".into()),
                usage: Some(0.45),
            },
            PortStatus {
                index: 2,
                name: "Thunderbolt 4 (Left 1)".into(),
                kind: PortKind::Thunderbolt,
                side: Side::Left,
                connected: Some(true),
                device: Some("Apple Studio Display".into()),
                speed: Some("40 Gb/s".into()),
                power: Some("96W Host Power".into()),
                usage: Some(0.68),
            },
            PortStatus {
                index: 3,
                name: "Thunderbolt 4 (Left 2)".into(),
                kind: PortKind::Thunderbolt,
                side: Side::Left,
                connected: Some(false),
                device: None,
                speed: Some("Up to 40 Gb/s".into()),
                power: None,
                usage: None,
            },
            PortStatus {
                index: 4,
                name: "Audio Jack".into(),
                kind: PortKind::Headphone,
                side: Side::Left,
                connected: Some(false),
                device: None,
                speed: Some("3.5mm DAC".into()),
                power: None,
                usage: None,
            },
            PortStatus {
                index: 5,
                name: "HDMI 2.1".into(),
                kind: PortKind::Hdmi,
                side: Side::Right,
                connected: Some(false),
                device: None,
                speed: Some("Up to 8K 60Hz".into()),
                power: None,
                usage: None,
            },
            PortStatus {
                index: 6,
                name: "SDXC Card Reader".into(),
                kind: PortKind::SdCard,
                side: Side::Right,
                connected: Some(true),
                device: Some("SanDisk Extreme PRO 256GB".into()),
                speed: Some("UHS-II (312 MB/s)".into()),
                power: None,
                usage: Some(0.25),
            },
            PortStatus {
                index: 7,
                name: "Thunderbolt 4 (Right)".into(),
                kind: PortKind::Thunderbolt,
                side: Side::Right,
                connected: Some(false),
                device: None,
                speed: Some("Up to 40 Gb/s".into()),
                power: None,
                usage: None,
            },
        ];

        Snapshot {
            seq: self.seq,
            host: HostInfo {
                hostname: Some("macbook-pro.local".into()),
                os: Some("macOS 15.6 Sequoia".into()),
                kernel: Some("Darwin 24.6.0".into()),
                uptime_secs: Some(3 * 86_400 + 4 * 3600 + 22 * 60),
                load_avg: Some([1.84, 2.12, 1.95]),
                process_count: 342,
            },
            cpu: CpuStats {
                brand: Some("Apple M3 Pro".into()),
                usage: curr_cpu,
                cores,
                history: cpu_history,
                frequency_mhz: Some(4050),
            },
            memory: MemoryStats {
                total: 36 * 1024 * 1024 * 1024,
                used: 19 * 1024 * 1024 * 1024,
                available: 17 * 1024 * 1024 * 1024,
                swap_total: 4 * 1024 * 1024 * 1024,
                swap_used: 120 * 1024 * 1024,
                pressure: Some(22.0),
                history: mem_history,
            },
            network: NetworkStats {
                interfaces,
                rx_rate: curr_rx,
                tx_rate: curr_tx,
                rx_history,
                tx_history,
            },
            processes,
            disks: DiskStats {
                volumes,
                io: DiskIo {
                    read_rate: curr_dr,
                    write_rate: curr_dw,
                    read_history: disk_read_history,
                    write_history: disk_write_history,
                    iops: Some(1420.0),
                    latency_ms: Some(0.28),
                    queue_depth: Some(1.2),
                    busy_pct: Some(14.5),
                },
                breakdowns: vec![breakdown],
                smart,
            },
            sensors: SensorStats {
                temps: vec![
                    TempSensor {
                        label: "CPU P-Cores".into(),
                        celsius: 48.0,
                        history: temp_history.clone(),
                    },
                    TempSensor {
                        label: "CPU E-Cores".into(),
                        celsius: 41.0,
                        history: temp_history.iter().map(|t| t - 6.0).collect(),
                    },
                    TempSensor {
                        label: "GPU Shader Core".into(),
                        celsius: 44.0,
                        history: temp_history.iter().map(|t| t - 3.0).collect(),
                    },
                    TempSensor {
                        label: "Apple SSD NAND".into(),
                        celsius: 35.0,
                        history: temp_history.iter().map(|t| t - 11.0).collect(),
                    },
                    TempSensor {
                        label: "Battery Pack".into(),
                        celsius: 31.0,
                        history: temp_history.iter().map(|t| t - 15.0).collect(),
                    },
                ],
                fans: vec![
                    Fan {
                        label: "Left Blower".into(),
                        rpm: 1450,
                        max_rpm: Some(5800),
                    },
                    Fan {
                        label: "Right Blower".into(),
                        rpm: 1480,
                        max_rpm: Some(5800),
                    },
                ],
            },
            battery: Some(BatteryInfo {
                percent: 84.0,
                state: ChargeState::Discharging,
                source: Some(PowerSource::Battery),
                power_w: Some(18.4),
                time_to_full_secs: None,
                time_to_empty_secs: Some(6 * 3600 + 15 * 60),
                cycle_count: Some(42),
                health_pct: Some(99.0),
                condition: Some("Normal".into()),
                adapter_watts: None,
            }),
            hardware: HardwareInfo {
                model_name: Some("MacBook Pro 14-inch".into()),
                model_id: Some("Mac15,6".into()),
                chip: Some("Apple M3 Pro (12 CPU, 18 GPU)".into()),
                cores: Some(CoreCount {
                    total: 12,
                    performance: Some(6),
                    efficiency: Some(6),
                }),
                gpu: Some("Apple M3 Pro GPU".into()),
                gpu_cores: Some(18),
                memory_total: Some(36 * 1024 * 1024 * 1024),
                display: Some("Liquid Retina XDR (3024 x 1964 @ 120Hz ProMotion)".into()),
                wifi: Some(WifiInfo {
                    interface: "en0".into(),
                    ssid: Some("HomeLab-5G".into()),
                    link_mbps: Some(1200.0),
                    signal_dbm: Some(-52),
                    noise_dbm: Some(-92),
                    channel: Some("36 (5 GHz, 80 MHz)".into()),
                    phy_mode: Some("802.11ax (Wi-Fi 6)".into()),
                }),
                bluetooth_on: Some(true),
                bluetooth: vec![
                    BluetoothDevice {
                        name: "AirPods Pro 2".into(),
                        kind: Some("Headphones".into()),
                        connected: true,
                    },
                    BluetoothDevice {
                        name: "Magic Keyboard".into(),
                        kind: Some("Keyboard".into()),
                        connected: true,
                    },
                    BluetoothDevice {
                        name: "MX Master 3S".into(),
                        kind: Some("Mouse".into()),
                        connected: true,
                    },
                ],
                storage: vec![StorageDevice {
                    name: "APPLE SSD AP0512R".into(),
                    interface: Some("PCIe 4.0 x4 NVMe".into()),
                    health: HealthStatus::Verified,
                    size: Some(500_277_792_768),
                    read_rate: Some(curr_dr),
                    write_rate: Some(curr_dw),
                }],
            },
            ports: PortStats {
                layout_name: "MacBook Pro 14-inch (M3 Pro/Max, 2023)".into(),
                known_layout: true,
                form: DeviceForm::Laptop,
                ports,
            },
        }
    }
}
