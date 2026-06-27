//! IBus bus discovery and connection bootstrap.
//!
//! IBus is a private D-Bus bus. We discover its address (preferring the
//! `IBUS_ADDRESS` env var the daemon sets when it launches us, falling back to
//! the per-session socket file), connect with zbus, request the well-known name
//! `org.freedesktop.IBus.Kime`, and serve the factory object.

use std::error::Error;
use std::path::PathBuf;
use std::sync::Arc;

use kime_engine_core::load_engine_config_from_config_dir;

use crate::factory::IBusFactory;

const WELL_KNOWN_NAME: &str = "org.freedesktop.IBus.Kime";
const FACTORY_PATH: &str = "/org/freedesktop/IBus/Factory";

/// Connect to the IBus daemon, register the factory and run until disconnect.
pub async fn run() -> Result<(), Box<dyn Error>> {
    let address = get_ibus_address()?;
    log::info!("Connecting to IBus at {}", address);

    let conn = zbus::connection::Builder::address(address.as_str())?
        .build()
        .await?;

    let config = Arc::new(load_engine_config_from_config_dir().unwrap_or_default());

    let factory = IBusFactory::new(conn.clone(), config);
    conn.object_server().at(FACTORY_PATH, factory).await?;

    conn.request_name(WELL_KNOWN_NAME).await?;
    log::info!(
        "Registered {} and factory at {}",
        WELL_KNOWN_NAME,
        FACTORY_PATH
    );

    // The object server runs on zbus's internal tasks; keep the process alive so
    // it can serve method calls. ibus-daemon will respawn us as needed.
    std::future::pending::<()>().await;

    Ok(())
}

/// Discover the IBus bus address.
fn get_ibus_address() -> Result<String, Box<dyn Error>> {
    if let Ok(addr) = std::env::var("IBUS_ADDRESS") {
        if !addr.is_empty() {
            return Ok(addr);
        }
    }

    let path = socket_file_path()?;
    log::debug!("Reading IBus address from {}", path.display());
    let content = std::fs::read_to_string(&path)?;
    for line in content.lines() {
        if let Some(rest) = line.strip_prefix("IBUS_ADDRESS=") {
            return Ok(rest.trim().to_string());
        }
    }

    Err(format!("IBUS_ADDRESS not found in {}", path.display()).into())
}

/// `$XDG_CONFIG_HOME/ibus/bus/<machine-id>-<host>-<display>` (or `~/.config/...`).
fn socket_file_path() -> Result<PathBuf, Box<dyn Error>> {
    let machine_id = read_machine_id()?;
    let (host, display) = display_parts();

    let config_dir = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .filter(|p| !p.as_os_str().is_empty())
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))
        .ok_or("Neither XDG_CONFIG_HOME nor HOME is set")?;

    Ok(config_dir
        .join("ibus")
        .join("bus")
        .join(format!("{machine_id}-{host}-{display}")))
}

fn read_machine_id() -> Result<String, Box<dyn Error>> {
    for path in ["/etc/machine-id", "/var/lib/dbus/machine-id"] {
        if let Ok(content) = std::fs::read_to_string(path) {
            let id = content.trim();
            if !id.is_empty() {
                return Ok(id.to_string());
            }
        }
    }
    Err("Could not read machine-id".into())
}

/// Parse `$DISPLAY` into the host/display-number components IBus uses for its
/// socket file name. For `:0` this yields `("unix", "0")`.
fn display_parts() -> (String, String) {
    let display = std::env::var("DISPLAY").unwrap_or_default();
    match display.rsplit_once(':') {
        Some((host, tail)) => {
            let host = if host.is_empty() {
                "unix".to_string()
            } else {
                host.to_string()
            };
            let number = tail.split('.').next().unwrap_or("0").to_string();
            (host, number)
        }
        // No DISPLAY (e.g. pure Wayland session): the env-var path is normally
        // used instead, but fall back to the common default.
        None => ("unix".to_string(), "0".to_string()),
    }
}
