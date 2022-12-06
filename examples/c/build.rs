use minicbor_bindgen::{Language, Options};
use std::{env, fs, path::PathBuf};

fn main() {
    // Generate mcbor bindings
    let opts = Options {
        language: Language::C,
        prefix: None,
        version: None,
    };
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let cddl = fs::read_to_string(root.parent().unwrap().join("data.cddl")).unwrap();
    let lib = minicbor_bindgen::render_lib(&cddl, &opts).unwrap();
    let out = PathBuf::from(env::var("OUT_DIR").unwrap()).join("bindings.rs");
    fs::write(out, lib).expect("failed to write bindings");

    // Generate C bindings
    // let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    // cbindgen::Builder::new()
    //     .with_crate(crate_dir)
    //     .with_language(cbindgen::Language::C)
    //     .with_parse_expand(&["c"])
    //     .generate()
    //     .expect("Unable to generate bindings")
    //     .write_to_file("bindings.h");
}
