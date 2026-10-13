//! Data collector interface and orchestration.

pub mod collector;
pub mod demo;
#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(target_os = "macos")]
pub mod macos;

use crate::model::{Needs, Snapshot};

/// Every platform data source implements `Collector`.
pub trait Collector {
    /// Sample system metrics based on current page needs.
    fn sample(&mut self, needs: &Needs) -> Snapshot;

    /// Force a rescan of slow hardware devices (e.g. system_profiler or sysfs).
    fn rescan_hardware(&mut self) {}
}
