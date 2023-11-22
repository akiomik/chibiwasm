use chibiwasm::{wasi::WasiSnapshotPreview1, Runtime};

fn main() -> anyhow::Result<()> {
    let wasi = WasiSnapshotPreview1::default();
    let mut runtime = Runtime::from_file("examples/fd_read.wasm", Some(Box::new(wasi)))?;
    runtime.call("_start".into(), vec![])?;
    Ok(())
}
