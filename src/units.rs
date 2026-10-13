//! Human-readable formatting for bytes, rates, durations and temperatures.
//!
//! Disk sizes and transfer rates use decimal (SI) units like Finder and drive
//! vendors do; memory uses binary units (GiB) like the kernel reports it.

const SI: [&str; 6] = ["B", "kB", "MB", "GB", "TB", "PB"];
const BIN: [&str; 6] = ["B", "KiB", "MiB", "GiB", "TiB", "PiB"];

fn scaled(value: f64, base: f64, units: &[&str]) -> String {
    if !value.is_finite() || value <= 0.0 {
        return format!("0 {}", units[0]);
    }
    let mut v = value;
    let mut i = 0;
    while v >= base && i < units.len() - 1 {
        v /= base;
        i += 1;
    }
    if i == 0 {
        format!("{v:.0} {}", units[0])
    } else if v >= 100.0 {
        format!("{v:.0} {}", units[i])
    } else if v >= 10.0 {
        format!("{v:.1} {}", units[i])
    } else {
        format!("{v:.2} {}", units[i])
    }
}

/// Decimal bytes: `1_500_000` -> `"1.50 MB"`.
pub fn bytes(n: u64) -> String {
    scaled(n as f64, 1000.0, &SI)
}

/// Binary bytes: `1 << 30` -> `"1.00 GiB"`.
pub fn bytes_bin(n: u64) -> String {
    scaled(n as f64, 1024.0, &BIN)
}

/// Transfer rate in decimal bytes per second: `"12.3 MB/s"`.
pub fn rate(bytes_per_sec: f64) -> String {
    format!("{}/s", scaled(bytes_per_sec, 1000.0, &SI))
}

/// Compact duration: `"3d 4h"`, `"4h 12m"`, `"12m"`, `"45s"`.
pub fn duration(secs: u64) -> String {
    let d = secs / 86_400;
    let h = (secs % 86_400) / 3600;
    let m = (secs % 3600) / 60;
    if d > 0 {
        format!("{d}d {h}h")
    } else if h > 0 {
        format!("{h}h {m:02}m")
    } else if m > 0 {
        format!("{m}m")
    } else {
        format!("{secs}s")
    }
}

/// Temperature in the chosen unit, no decimals: `"54°C"` / `"129°F"`.
pub fn temp(celsius: f32, fahrenheit: bool) -> String {
    if fahrenheit {
        format!("{:.0}°F", celsius * 9.0 / 5.0 + 32.0)
    } else {
        format!("{celsius:.0}°C")
    }
}

/// Percentage without decimals: `"42%"`.
pub fn percent(p: f64) -> String {
    if p.is_finite() {
        format!("{:.0}%", p.clamp(0.0, 999.0))
    } else {
        "n/a".to_string()
    }
}

/// Large counts with thousands separators: `12345` -> `"12,345"`.
pub fn count(n: u64) -> String {
    let s = n.to_string();
    let mut out = String::with_capacity(s.len() + s.len() / 3);
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    out
}

/// Truncates to `max` characters (not bytes), appending `…` when cut.
pub fn truncate(s: &str, max: usize) -> String {
    if max == 0 {
        return String::new();
    }
    if s.chars().count() <= max {
        return s.to_string();
    }
    let mut out: String = s.chars().take(max.saturating_sub(1)).collect();
    out.push('…');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decimal_bytes() {
        assert_eq!(bytes(0), "0 B");
        assert_eq!(bytes(999), "999 B");
        assert_eq!(bytes(1_500_000), "1.50 MB");
        assert_eq!(bytes(15_000_000), "15.0 MB");
        assert_eq!(bytes(494_380_000_000), "494 GB");
        assert_eq!(bytes(2_000_000_000_000), "2.00 TB");
    }

    #[test]
    fn binary_bytes() {
        assert_eq!(bytes_bin(1 << 30), "1.00 GiB");
        assert_eq!(bytes_bin(18 * (1 << 30)), "18.0 GiB");
        assert_eq!(bytes_bin(512), "512 B");
    }

    #[test]
    fn rates() {
        assert_eq!(rate(0.0), "0 B/s");
        assert_eq!(rate(f64::NAN), "0 B/s");
        assert_eq!(rate(-5.0), "0 B/s");
        assert_eq!(rate(12_300_000.0), "12.3 MB/s");
    }

    #[test]
    fn durations() {
        assert_eq!(duration(45), "45s");
        assert_eq!(duration(12 * 60), "12m");
        assert_eq!(duration(4 * 3600 + 5 * 60), "4h 05m");
        assert_eq!(duration(3 * 86_400 + 4 * 3600), "3d 4h");
    }

    #[test]
    fn temps() {
        assert_eq!(temp(54.4, false), "54°C");
        assert_eq!(temp(100.0, true), "212°F");
    }

    #[test]
    fn counts_and_truncation() {
        assert_eq!(count(0), "0");
        assert_eq!(count(1234567), "1,234,567");
        assert_eq!(truncate("hello", 10), "hello");
        assert_eq!(truncate("hello world", 6), "hello…");
        assert_eq!(truncate("héllo", 2), "h…");
        assert_eq!(truncate("x", 0), "");
        assert_eq!(percent(42.4), "42%");
    }
}
