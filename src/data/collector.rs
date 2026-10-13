//! Common data collector using `sysinfo` with delta rate calculations and ring buffers.

use std::collections::HashMap;
use std::time::Instant;

use sysinfo::{
    CpuRefreshKind, DiskRefreshKind, Disks, MemoryRefreshKind, Networks,
    ProcessRefreshKind, ProcessesToUpdate, RefreshKind, System, UpdateKind,
};

use crate::data::Collector;
use crate::model::*;

pub struct SystemCollector {
    sys: System,
    disks: Disks,
    networks: Networks,
    last_sample: Instant,

    // Previous totals for delta rate calculations
    prev_net_rx: HashMap<String, u64>,
    prev_net_tx: HashMap<String, u64>,
    prev_disk_read: u64,
    prev_disk_write: u64,
    prev_proc_disk_read: HashMap<u32, u64>,
    prev_proc_disk_write: HashMap<u32, u64>,

    // Rolling 60-sample history buffers
    cpu_history: Vec<f32>,
    mem_history: Vec<f32>,
    net_rx_history: Vec<f64>,
    net_tx_history: Vec<f64>,
    disk_read_history: Vec<f64>,
    disk_write_history: Vec<f64>,

    seq: u64,
}

impl SystemCollector {
    pub fn new() -> Self {
        let mut sys = System::new_with_specifics(
            RefreshKind::nothing()
                .with_cpu(CpuRefreshKind::nothing().with_cpu_usage().with_frequency())
                .with_memory(MemoryRefreshKind::everything()),
        );
        sys.refresh_cpu_all();
        sys.refresh_memory();

        let disks = Disks::new_with_refreshed_list_specifics(DiskRefreshKind::everything());
        let networks = Networks::new_with_refreshed_list();

        Self {
            sys,
            disks,
            networks,
            last_sample: Instant::now(),

            prev_net_rx: HashMap::new(),
            prev_net_tx: HashMap::new(),
            prev_disk_read: 0,
            prev_disk_write: 0,
            prev_proc_disk_read: HashMap::new(),
            prev_proc_disk_write: HashMap::new(),

            cpu_history: Vec::with_capacity(HISTORY_LEN),
            mem_history: Vec::with_capacity(HISTORY_LEN),
            net_rx_history: Vec::with_capacity(HISTORY_LEN),
            net_tx_history: Vec::with_capacity(HISTORY_LEN),
            disk_read_history: Vec::with_capacity(HISTORY_LEN),
            disk_write_history: Vec::with_capacity(HISTORY_LEN),

            seq: 0,
        }
    }
}

impl Default for SystemCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl Collector for SystemCollector {
    fn sample(&mut self, needs: &Needs) -> Snapshot {
        self.seq += 1;
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_sample).as_secs_f64().max(0.001);
        self.last_sample = now;

        // 1. Refresh CPU and Memory
        self.sys.refresh_cpu_usage();
        self.sys.refresh_memory();

        let global_cpu = self.sys.global_cpu_usage();
        push_history(&mut self.cpu_history, global_cpu);

        let mem_used = self.sys.used_memory();
        let mem_total = self.sys.total_memory();
        let mem_pct = if mem_total > 0 {
            (mem_used as f32 / mem_total as f32) * 100.0
        } else {
            0.0
        };
        push_history(&mut self.mem_history, mem_pct);

        let brand = self.sys.cpus().first().map(|c| c.brand().to_string());
        let freq = self.sys.cpus().first().map(|c| c.frequency());

        let mut cores = Vec::with_capacity(self.sys.cpus().len());
        for (i, c) in self.sys.cpus().iter().enumerate() {
            cores.push(CoreUsage {
                label: format!("C{i}"),
                kind: CoreKind::Generic,
                usage: c.cpu_usage(),
            });
        }

        // 2. Refresh Networks
        self.networks.refresh(true);
        let mut iface_stats = Vec::new();
        let mut total_rx_rate = 0.0;
        let mut total_tx_rate = 0.0;

        for (name, data) in self.networks.list() {
            let rx_bytes = data.total_received();
            let tx_bytes = data.total_transmitted();

            let prev_rx = self.prev_net_rx.insert(name.clone(), rx_bytes).unwrap_or(rx_bytes);
            let prev_tx = self.prev_net_tx.insert(name.clone(), tx_bytes).unwrap_or(tx_bytes);

            let rx_rate = rx_bytes.saturating_sub(prev_rx) as f64 / elapsed;
            let tx_rate = tx_bytes.saturating_sub(prev_tx) as f64 / elapsed;

            let is_loopback = name.starts_with("lo");
            if !is_loopback {
                total_rx_rate += rx_rate;
                total_tx_rate += tx_rate;
            }

            let kind = if name.starts_with("en0") || name.starts_with("wlan") || name.starts_with("wl") {
                IfaceKind::Wifi
            } else if is_loopback {
                IfaceKind::Loopback
            } else if name.starts_with("eth") || name.starts_with("en") {
                IfaceKind::Ethernet
            } else {
                IfaceKind::Other
            };

            let mac = Some(data.mac_address().to_string());
            let addresses = data.ip_networks().iter().map(|ip| ip.addr.to_string()).collect();

            iface_stats.push(InterfaceStats {
                name: name.clone(),
                kind,
                rx_rate,
                tx_rate,
                rx_total: rx_bytes,
                tx_total: tx_bytes,
                rx_history: vec![rx_rate; HISTORY_LEN],
                tx_history: vec![tx_rate; HISTORY_LEN],
                mac,
                addresses,
            });
        }
        iface_stats.sort_by(|a, b| a.name.cmp(&b.name));
        push_history(&mut self.net_rx_history, total_rx_rate);
        push_history(&mut self.net_tx_history, total_tx_rate);

        // 3. Refresh Processes (if requested by current page)
        let mut processes = Vec::new();
        let total_proc_count;
        if needs.processes {
            let mut refresh_kind = ProcessRefreshKind::default()
                .with_cpu()
                .with_memory();

            if needs.process_details {
                refresh_kind = refresh_kind
                    .with_cmd(UpdateKind::OnlyIfNotSet)
                    .with_user(UpdateKind::OnlyIfNotSet);
            }

            self.sys.refresh_processes_specifics(
                ProcessesToUpdate::All,
                true,
                refresh_kind,
            );

            total_proc_count = self.sys.processes().len();

            for (&pid, p) in self.sys.processes() {
                let pid_u32 = pid.as_u32();
                let disk_usage = p.disk_usage();
                let r_bytes = disk_usage.total_read_bytes;
                let w_bytes = disk_usage.total_written_bytes;

                let prev_r = self.prev_proc_disk_read.insert(pid_u32, r_bytes).unwrap_or(r_bytes);
                let prev_w = self.prev_proc_disk_write.insert(pid_u32, w_bytes).unwrap_or(w_bytes);

                let r_rate = r_bytes.saturating_sub(prev_r) as f64 / elapsed;
                let w_rate = w_bytes.saturating_sub(prev_w) as f64 / elapsed;

                let cmd_str = p.cmd().iter().map(|c| c.to_string_lossy().to_string()).collect::<Vec<_>>().join(" ");
                let name = p.name().to_string_lossy().to_string();

                processes.push(ProcessInfo {
                    pid: pid_u32,
                    name,
                    user: None,
                    cpu: p.cpu_usage(),
                    memory: p.memory(),
                    threads: p.tasks().map(|t| t.len() as u32),
                    state: format!("{:?}", p.status()).chars().next().unwrap_or('?').to_string(),
                    command: cmd_str,
                    disk_read_rate: r_rate,
                    disk_write_rate: w_rate,
                });
            }
            // Sort by CPU usage descending by default
            processes.sort_by(|a, b| b.cpu.partial_cmp(&a.cpu).unwrap_or(std::cmp::Ordering::Equal));
        } else {
            total_proc_count = 0;
        }

        // 4. Refresh Disks
        let mut volumes = Vec::new();
        let mut tot_disk_read = 0;
        let mut tot_disk_write = 0;

        if needs.disks {
            self.disks.refresh(true);
            for d in self.disks.list() {
                let mount = d.mount_point().to_string_lossy().to_string();
                // Filter out non-physical, devfs, tmpfs, loops and duplicate APFS synthetic mounts
                if mount.starts_with("/dev")
                    || mount.starts_with("/System/Volumes/Data")
                    || mount.starts_with("/System/Volumes/VM")
                    || mount.starts_with("/System/Volumes/Preboot")
                    || mount.starts_with("/System/Volumes/Update")
                    || mount.contains("snap")
                {
                    continue;
                }

                let d_usage = d.usage();
                tot_disk_read += d_usage.total_read_bytes;
                tot_disk_write += d_usage.total_written_bytes;

                volumes.push(Volume {
                    name: d.name().to_string_lossy().to_string(),
                    device: mount.clone(),
                    mount,
                    fs: d.file_system().to_string_lossy().to_string(),
                    total: d.total_space(),
                    available: d.available_space(),
                    removable: d.is_removable(),
                    kind: match d.kind() {
                        sysinfo::DiskKind::SSD => "SSD".into(),
                        sysinfo::DiskKind::HDD => "HDD".into(),
                        _ => "Unknown".into(),
                    },
                });
            }
        }

        let disk_r_rate = tot_disk_read.saturating_sub(self.prev_disk_read) as f64 / elapsed;
        let disk_w_rate = tot_disk_write.saturating_sub(self.prev_disk_write) as f64 / elapsed;
        self.prev_disk_read = tot_disk_read;
        self.prev_disk_write = tot_disk_write;

        push_history(&mut self.disk_read_history, disk_r_rate);
        push_history(&mut self.disk_write_history, disk_w_rate);

        let load = System::load_average();

        Snapshot {
            seq: self.seq,
            host: HostInfo {
                hostname: System::host_name(),
                os: System::long_os_version(),
                kernel: System::kernel_version(),
                uptime_secs: Some(System::uptime()),
                load_avg: Some([load.one, load.five, load.fifteen]),
                process_count: total_proc_count,
            },
            cpu: CpuStats {
                brand,
                usage: global_cpu,
                cores,
                history: self.cpu_history.clone(),
                frequency_mhz: freq,
            },
            memory: MemoryStats {
                total: mem_total,
                used: mem_used,
                available: self.sys.available_memory(),
                swap_total: self.sys.total_swap(),
                swap_used: self.sys.used_swap(),
                pressure: None,
                history: self.mem_history.clone(),
            },
            network: NetworkStats {
                interfaces: iface_stats,
                rx_rate: total_rx_rate,
                tx_rate: total_tx_rate,
                rx_history: self.net_rx_history.clone(),
                tx_history: self.net_tx_history.clone(),
            },
            processes,
            disks: DiskStats {
                volumes,
                io: DiskIo {
                    read_rate: disk_r_rate,
                    write_rate: disk_w_rate,
                    read_history: self.disk_read_history.clone(),
                    write_history: self.disk_write_history.clone(),
                    iops: None,
                    latency_ms: None,
                    queue_depth: None,
                    busy_pct: None,
                },
                breakdowns: Vec::new(),
                smart: Vec::new(),
            },
            sensors: SensorStats::default(),
            battery: None,
            hardware: HardwareInfo::default(),
            ports: PortStats::default(),
        }
    }
}

fn push_history<T: Copy>(vec: &mut Vec<T>, val: T) {
    if vec.len() >= HISTORY_LEN {
        vec.remove(0);
    }
    vec.push(val);
}
