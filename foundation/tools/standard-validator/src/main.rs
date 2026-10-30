//! Uses the pristine Horizon stock native bundle parser; never executes mod source.
use std::path::Path;
fn main() {
    let path = std::env::args().nth(1).expect("standard-validator package.hnmod");
    let path = Path::new(&path);
    let bundle = horizon_mods::index::native::Bundle::read(path)
        .unwrap_or_else(|error| panic!("Stock .hnmod validation failed: {error}"));
    assert!(horizon_mods::Package::read(path).is_err(), "Content loader accepted native code");
    println!("{}", serde_json::json!({
        "ok": true,
        "parser": "pristine horizon_mods::index::native::Bundle::read",
        "id": bundle.manifest.id,
        "version": bundle.manifest.version,
        "horizon": bundle.manifest.horizon,
        "mixins": bundle.manifest.mixins,
        "hooks": bundle.manifest.hooks,
        "entries": bundle.files.len(),
        "content_loader_rejected": true,
    }));
}
