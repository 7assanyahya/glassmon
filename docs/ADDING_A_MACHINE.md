# Adding a Machine Port Layout

`glassmon` features a visual chassis inspector on the **Device** page that renders numbered port badges alongside a machine layout diagram. This guide describes how to add a port map for a new computer model.

---

## Machine Port Definition Structure

Port definitions are stored in `src/data/macos.rs` (and `src/data/linux.rs` for Linux machines) within the `map_mac_ports` / `map_linux_ports` functions.

Each port is defined using the `PortStatus` struct:

```rust
pub struct PortStatus {
    pub index: u8,               // 1-based index (matched to number keys 1-7)
    pub name: String,            // Human-readable label (e.g. "MagSafe 3", "Thunderbolt 4")
    pub kind: PortKind,          // Port connector type
    pub side: Side,              // Chassis placement: Left, Right, Back
    pub connected: Option<bool>, // Known connection state (Some(true), Some(false), None)
    pub device: Option<String>,  // Connected peripheral name
    pub speed: Option<String>,   // Rated bandwidth or charging capacity
    pub power: Option<String>,   // Power delivery specifications
    pub usage: Option<f32>,      // Link utilization ratio (0.0..=1.0)
}
```

### Supported Port Kinds

- `PortKind::Thunderbolt` (`TB`)
- `PortKind::UsbC` (`USB-C`)
- `PortKind::UsbA` (`USB-A`)
- `PortKind::MagSafe` (`MagSafe`)
- `PortKind::Hdmi` (`HDMI`)
- `PortKind::SdCard` (`SD`)
- `PortKind::Headphone` (`Audio`)
- `PortKind::Ethernet` (`LAN`)
- `PortKind::DisplayPort` (`DP`)
- `PortKind::Other`

---

## Example: Adding MacBook Air 13" (M2/M3)

Model identifiers: `Mac14,2`, `Mac15,2`, `Mac15,12`.

```rust
if id_lower.contains("mac14,2") || id_lower.contains("mac15,2") {
    return PortStats {
        layout_name: format!("{model_id} (MacBook Air 13-inch)"),
        known_layout: true,
        form: DeviceForm::Laptop,
        ports: vec![
            PortStatus {
                index: 1,
                name: "MagSafe 3".into(),
                kind: PortKind::MagSafe,
                side: Side::Left,
                connected: Some(true),
                device: Some("35W Dual USB-C Adapter".into()),
                speed: Some("35W Fast Charge".into()),
                power: Some("35W".into()),
                usage: Some(0.35),
            },
            PortStatus {
                index: 2,
                name: "Thunderbolt / USB 4 (Port 1)".into(),
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
                name: "Thunderbolt / USB 4 (Port 2)".into(),
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
                name: "3.5mm Headphone Jack".into(),
                kind: PortKind::Headphone,
                side: Side::Right,
                connected: None,
                device: None,
                speed: Some("High-Impedance Audio".into()),
                power: None,
                usage: None,
            },
        ],
    };
}
```

---

## Verifying Your Layout

1. Add your model identifier check to `map_mac_ports`.
2. Run tests to confirm it builds cleanly:
   ```bash
   cargo test
   ```
3. Test locally in demo mode or by mocking your model string.
4. Open the **Device** page (`6`), inspect the chassis outline and port list, and press `1`-`4` to verify port highlighting.
