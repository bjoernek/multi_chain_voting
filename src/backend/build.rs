use ic_cdk_bindgen::{Builder, Config};
use std::path::PathBuf;

fn main() {
    let manifest_dir =
        PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("Cannot find manifest dir"));
    let mut evm_rpc = Config::new("evm_rpc");
    evm_rpc
        .binding
        .set_type_attributes("#[derive(Debug, CandidType, Deserialize)]".into());
    let mut builder = Builder::new();
    builder.add(evm_rpc);
    builder.build(Some(manifest_dir.join("src/declarations")));
}
