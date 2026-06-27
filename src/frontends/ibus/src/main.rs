#[tokio::main]
async fn main() {
    // Handles --help/--version/--log and sets up logging; returns the parsed args.
    let mut args = kime_version::cli_boilerplate!((),);
    // Launched by ibus-daemon as `kime-ibus --ibus`; accept and ignore the flag.
    let _ = args.contains("--ibus");

    if let Err(e) = kime_ibus::run().await {
        log::error!("kime-ibus exited with error: {}", e);
        std::process::exit(1);
    }
}
