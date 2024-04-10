use dotenv::dotenv;
use ic_cdk_bindgen::{Builder, Config};
use std::path::PathBuf;

fn main() {
    dotenv().ok();
    let manifest_dir =
        PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("Cannot find manifest dir"));
    let mut builder = Builder::new();

    // ic_siwe_provider
    let mut ic_siwe_provider = Config::new("ic_siwe_provider");
    ic_siwe_provider
        .binding
        .set_type_attributes("#[derive(Debug, CandidType, Deserialize)]".into());
    builder.add(ic_siwe_provider);

    // evm_rpc
    let mut evm_rpc = Config::new("evm_rpc");
    evm_rpc
        .binding
        .set_type_attributes("#[derive(Debug, CandidType, Deserialize)]".into());
    builder.add(evm_rpc);

    builder.build(Some(manifest_dir.join("src/declarations")));
}
