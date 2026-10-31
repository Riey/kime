//! `WAYLAND_DEBUG=1` trace parsing for protocol-level assertions
//! (e.g. W-01/#714: no spurious `zwp_input_method_v2.commit` after a lone
//! modifier).
//!
//! Trace line shape (client-side request): `[1234.567] -> zwp_input_method_v2@15.commit(3)`
//! Events use `<-` (or no arrow on older libwayland); [`count_requests`]
//! counts outgoing requests only.

use std::path::Path;

/// Byte-offset marker into a growing trace file: everything after the marker
/// happened after [`marker`] was called.
#[derive(Debug, Clone, Copy)]
pub struct Marker(pub u64);

/// Take a marker at the current end of `trace` (0 if the file doesn't exist yet).
pub fn marker(trace: &Path) -> Marker {
    Marker(std::fs::metadata(trace).map(|m| m.len()).unwrap_or(0))
}

/// Trace text appended after `m` (lossy UTF-8; may start mid-line — the
/// request-counting helpers are substring-based and tolerate that).
pub fn text_after(trace: &Path, m: Marker) -> String {
    let Ok(bytes) = std::fs::read(trace) else {
        return String::new();
    };
    let start = (m.0 as usize).min(bytes.len());
    String::from_utf8_lossy(&bytes[start..]).into_owned()
}

/// Count outgoing `<interface>#N.<request>(` lines in `text` (older
/// libwayland prints `@` instead of `#`; both are accepted).
pub fn count_requests(text: &str, interface: &str, request: &str) -> usize {
    let iface_hash = format!("{interface}#");
    let iface_at = format!("{interface}@");
    let req_call = format!(".{request}(");
    text.lines()
        .filter(|l| {
            l.contains(" -> ")
                && (l.contains(&iface_hash) || l.contains(&iface_at))
                && l.contains(&req_call)
        })
        .count()
}

/// Count outgoing requests appended to `trace` after marker `m`.
pub fn count_requests_after(trace: &Path, m: Marker, interface: &str, request: &str) -> usize {
    count_requests(&text_after(trace, m), interface, request)
}
