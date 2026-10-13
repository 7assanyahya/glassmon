//! Application state, navigation, interaction modes, and event dispatch.

use crate::model::Needs;
use crate::ui::theme::{ColorCapability, Theme, ThemeMode};
use crate::ui::widgets::modal::ModalType;
use crate::ui::widgets::toast::{Toast, ToastKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Page {
    #[default]
    Overview,
    Processes,
    Network,
    Disks,
    Hardware,
    Device,
}

impl Page {
    pub fn next(self) -> Self {
        match self {
            Page::Overview => Page::Processes,
            Page::Processes => Page::Network,
            Page::Network => Page::Disks,
            Page::Disks => Page::Hardware,
            Page::Hardware => Page::Device,
            Page::Device => Page::Overview,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Page::Overview => Page::Device,
            Page::Processes => Page::Overview,
            Page::Network => Page::Processes,
            Page::Disks => Page::Network,
            Page::Hardware => Page::Disks,
            Page::Device => Page::Hardware,
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            Page::Overview => "System Overview",
            Page::Processes => "Process Monitor",
            Page::Network => "Network Interfaces",
            Page::Disks => "Storage & Disks",
            Page::Hardware => "Hardware & Sensors",
            Page::Device => "Device Architecture",
        }
    }

    pub fn subtitle(self) -> &'static str {
        match self {
            Page::Overview => "Live CPU, memory, network and system health",
            Page::Processes => "Real-time process tree, resources, search and signal management",
            Page::Network => "Per-interface throughput, traffic counters and IP configuration",
            Page::Disks => "Storage category breakdown, I/O latency, throughput and SMART metrics",
            Page::Hardware => "Thermals, fan speeds, power draw, display and connectivity",
            Page::Device => "Physical chassis port mapping, link speed and device health tiles",
        }
    }

    pub fn needs(self) -> Needs {
        match self {
            Page::Overview => Needs {
                processes: true,
                process_details: false,
                sensors: true,
                disks: true,
                disk_details: false,
                hardware: false,
                battery: false,
            },
            Page::Processes => Needs {
                processes: true,
                process_details: true,
                sensors: false,
                disks: false,
                disk_details: false,
                hardware: false,
                battery: false,
            },
            Page::Network => Needs {
                processes: false,
                process_details: false,
                sensors: false,
                disks: false,
                disk_details: false,
                hardware: false,
                battery: false,
            },
            Page::Disks => Needs {
                processes: true, // for top disk activity table
                process_details: false,
                sensors: false,
                disks: true,
                disk_details: true,
                hardware: false,
                battery: false,
            },
            Page::Hardware => Needs {
                processes: false,
                process_details: false,
                sensors: true,
                disks: false,
                disk_details: false,
                hardware: true,
                battery: true,
            },
            Page::Device => Needs {
                processes: false,
                process_details: false,
                sensors: true,
                disks: false,
                disk_details: false,
                hardware: true,
                battery: true,
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ProcessSort {
    #[default]
    Cpu,
    Memory,
    Pid,
    Name,
}

impl ProcessSort {
    pub fn cycle(self) -> Self {
        match self {
            ProcessSort::Cpu => ProcessSort::Memory,
            ProcessSort::Memory => ProcessSort::Pid,
            ProcessSort::Pid => ProcessSort::Name,
            ProcessSort::Name => ProcessSort::Cpu,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            ProcessSort::Cpu => "CPU%",
            ProcessSort::Memory => "Memory",
            ProcessSort::Pid => "PID",
            ProcessSort::Name => "Name",
        }
    }
}

pub struct App {
    pub page: Page,
    pub theme: Theme,
    pub paused: bool,
    pub fahrenheit: bool,
    pub should_quit: bool,

    // Process page state
    pub process_selected: usize,
    pub process_sort: ProcessSort,
    pub process_sort_reverse: bool,
    pub process_filter: String,
    pub filter_active: bool,

    // Network page state
    pub net_iface_index: usize,

    // Disks page state
    pub disk_selected: usize,
    pub disk_sort_desc: bool,

    // Device page state
    pub selected_port: Option<u8>,

    // Modals & Overlays
    pub active_modal: Option<ModalType>,
    pub toasts: Vec<Toast>,

    // Rescan flag
    pub request_rescan: bool,
}

impl App {
    pub fn new(mode: ThemeMode, cap: ColorCapability) -> Self {
        Self {
            page: Page::Overview,
            theme: Theme::new(mode, cap),
            paused: false,
            fahrenheit: false,
            should_quit: false,

            process_selected: 0,
            process_sort: ProcessSort::Cpu,
            process_sort_reverse: false,
            process_filter: String::new(),
            filter_active: false,

            net_iface_index: 0,
            disk_selected: 0,
            disk_sort_desc: true,
            selected_port: None,

            active_modal: None,
            toasts: Vec::new(),
            request_rescan: false,
        }
    }

    pub fn toggle_theme(&mut self) {
        let new_mode = match self.theme.mode {
            ThemeMode::Dark => ThemeMode::Light,
            ThemeMode::Light => ThemeMode::Dark,
        };
        self.theme = Theme::new(new_mode, self.theme.cap);
        self.add_toast(format!("Theme switched to {:?}", new_mode), ToastKind::Info);
    }

    pub fn toggle_temp_unit(&mut self) {
        self.fahrenheit = !self.fahrenheit;
        let u = if self.fahrenheit {
            "Fahrenheit (°F)"
        } else {
            "Celsius (°C)"
        };
        self.add_toast(format!("Temperature units: {u}"), ToastKind::Info);
    }

    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
        let msg = if self.paused {
            "Sampling paused"
        } else {
            "Sampling resumed"
        };
        self.add_toast(msg, ToastKind::Info);
    }

    pub fn add_toast(&mut self, msg: impl Into<String>, kind: ToastKind) {
        self.toasts.push(Toast::new(msg, kind));
        // Keep at most 3 toasts
        if self.toasts.len() > 3 {
            self.toasts.remove(0);
        }
    }

    pub fn clean_expired_toasts(&mut self) {
        self.toasts.retain(|t| !t.is_expired());
    }
}
