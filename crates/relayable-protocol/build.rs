//! Build script for `relayable-protocol`.
//!
//! Compiles the frozen CORE-001 schemas in `protocol/schema/` into OUT_DIR with
//! `prost-build`. `prost-build` shells out to the external `protoc` binary
//! (verified: `/usr/bin/protoc` = libprotoc 36.1); it is not bundled. The
//! `format` feature is disabled in the workspace dependency, so generated code
//! is written verbatim and never prettyprinted.

use std::path::PathBuf;

fn main() {
    let schema_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../protocol/schema");

    let protos = [
        "common.proto",
        "control.proto",
        "terminal.proto",
        "events.proto",
        "auth.proto",
        "admin.proto",
    ];
    let proto_paths: Vec<PathBuf> = protos.iter().map(|name| schema_dir.join(name)).collect();

    for path in &proto_paths {
        println!("cargo:rerun-if-changed={}", path.display());
    }
    println!("cargo:rerun-if-changed={}", schema_dir.display());

    prost_build::Config::new()
        .compile_protos(&proto_paths, &[&schema_dir])
        .expect("failed to compile protocol schemas in protocol/schema/");
}
