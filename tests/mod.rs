use minicbor_bindgen::{Language, Options};
use std::fs;
use std::path::PathBuf;

fn render_cddl(path: &str, opts: Options) -> () {
    let root = PathBuf::from(std::env!("CARGO_MANIFEST_DIR"));
    let cddl = fs::read_to_string(root.join("tests/thing.cddl")).unwrap();
    let mut lib = minicbor_bindgen::render_lib(&cddl, &opts).unwrap();
    lib.push_str(" fn main () {}");
    fs::write(root.join("tests").join(path), lib).unwrap();
}

#[test]
fn test_render_lib() {
    let runner = trybuild::TestCases::new();
    render_cddl(
        "__generated__/c_with_prefix.rs",
        Options {
            language: Language::C,
            prefix: Some("foo".into()),
            ..Options::default()
        },
    );
    render_cddl(
        "__generated__/c.rs",
        Options {
            language: Language::C,
            ..Options::default()
        },
    );
    render_cddl(
        "__generated__/rust_with_prefix.rs",
        Options {
            language: Language::Rust,
            prefix: Some("foo".into()),
            ..Options::default()
        },
    );
    render_cddl(
        "__generated__/rust.rs",
        Options {
            language: Language::Rust,
            ..Options::default()
        },
    );

    runner.pass("tests/__generated__/c.rs");
    runner.pass("tests/__generated__/c_with_prefix.rs");
    runner.pass("tests/__generated__/rust.rs");
    runner.pass("tests/__generated__/rust_with_prefix.rs");
}
