use super::util::*;
use crate::*;

#[test]
fn test_render_lib_full() {
    let expect = read_expect("lib.rs.expect");
    let cddl = read_cddl("thing.cddl");
    let test = render_lib(&cddl, &Options::default()).unwrap();
    expect.validate(&test);
}

#[test]
fn test_render_lib_bindings() {
    let options = Options {
        bindings: true,
        prefix: Some("foo_".into()),
        ..Options::default()
    };
    let expect = read_expect("lib.rs.bindings.expect");
    let cddl = read_cddl("thing.cddl");
    let test = render_lib(&cddl, &options).unwrap();
    expect.validate(&test);
}

#[test]
fn test_render_cargo_full() {
    let expect = read_expect("cargo.toml.expect");
    let test = render_cargo("test", &Options::default()).unwrap();
    expect.validate(&test);
}

#[test]
fn test_render_cargo_bindings() {
    let options = Options {
        bindings: true,
        version: Some("0.3.0".into()),
        ..Options::default()
    };
    let expect = read_expect("cargo.toml.bindings.expect");
    let test = render_cargo("test", &options).unwrap();
    expect.validate(&test);
}

#[test]
fn test_render_extra() {
    let options = Options {
        bindings: true,
        prefix: Some("foo_".into()),
        ..Options::default()
    };
    let expect = read_expect("extra.h.expect");
    let cddl = read_cddl("thing.cddl");
    let test = render_extra(&cddl, &options).unwrap();
    expect.validate_c(&test);
}
