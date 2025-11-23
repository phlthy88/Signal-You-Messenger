use std::env;
use std::path::Path;
use std::process::Command;

fn main() {
    // Compile GResource
    let out_dir = env::var("OUT_DIR").unwrap();
    let src_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    let gresource_xml = Path::new(&src_dir).join("src/signal-you-messenger.gresource.xml");
    let gresource_out = Path::new(&out_dir).join("signal-you-messenger.gresource");

    // Tell cargo to rerun if the gresource XML or any of its dependencies change
    println!("cargo:rerun-if-changed={}", gresource_xml.display());
    println!("cargo:rerun-if-changed=src/ui/window.ui");
    println!("cargo:rerun-if-changed=src/ui/chat_list.ui");
    println!("cargo:rerun-if-changed=src/ui/chat_view.ui");
    println!("cargo:rerun-if-changed=src/ui/compose_bar.ui");
    println!("cargo:rerun-if-changed=src/ui/message_row.ui");
    println!("cargo:rerun-if-changed=src/ui/contact_row.ui");
    println!("cargo:rerun-if-changed=src/ui/link_device_view.ui");
    println!("cargo:rerun-if-changed=src/style.css");

    // Compile the gresource
    let status = Command::new("glib-compile-resources")
        .arg("--sourcedir")
        .arg(Path::new(&src_dir).join("src"))
        .arg("--target")
        .arg(&gresource_out)
        .arg(&gresource_xml)
        .status()
        .expect("Failed to run glib-compile-resources. Make sure glib2.0-dev is installed.");

    if !status.success() {
        panic!("glib-compile-resources failed");
    }

    // Protobuf compilation (if needed)
    // prost_build::compile_protos(&["src/proto/signal.proto"], &["src/proto/"]).unwrap();
}
