use std::io::Write;

fn main() {
    let mut out = std::io::stdout();
    out.write_all(
        serde_yaml::to_string(&kime_config::RawConfig::default())
            .unwrap()
            .as_bytes(),
    )
    .expect("Write to stdout");
    out.flush().unwrap();
}
