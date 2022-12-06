use std::fs;
use std::io::prelude::*;
use std::path::PathBuf;

pub trait Validate {
    fn validate(&self, other: &Self) -> &Self;

    fn validate_c(&self, other: &Self) -> &Self;
}

impl Validate for str {
    fn validate(&self, other: &Self) -> &str {
        let mut a = self
            .to_string()
            .lines()
            .map(|line| line.trim_start().trim_end())
            .collect::<String>();
        let mut b = other
            .to_string()
            .lines()
            .map(|line| line.trim_start().trim_end())
            .collect::<String>();
        a.retain(|c| !(c == '\n'));
        b.retain(|c| !(c == '\n'));
        if !(a == b) {
            std::fs::write(".dump-a.rs", &a).unwrap();
            std::fs::write(".dump-b.rs", &b).unwrap();
        }
        assert_eq!(a, b);
        self
    }

    // TODO execute clang-format on string in place
    fn validate_c(&self, other: &Self) -> &Self {
        let a = self
            .to_string()
            .lines()
            .map(|line| line.trim_start().trim_end())
            .collect::<Vec<&str>>()
            .join("\n");
        let b = other
            .to_string()
            .lines()
            .map(|line| line.trim_start().trim_end())
            .collect::<Vec<&str>>()
            .join("\n");
        if !(a == b) {
            std::fs::write(".dump-a.h", &a).unwrap();
            std::fs::write(".dump-b.h", &b).unwrap();
        }
        assert_eq!(a, b);
        self
    }
}

pub fn read_cddl(path: &str) -> String {
    let path = PathBuf::from(std::env!("CARGO_MANIFEST_DIR"))
        .join("src/tests/cddl")
        .join(path);
    let mut file = fs::File::open(path).unwrap();
    let mut cddl = String::new();
    file.read_to_string(&mut cddl).unwrap();
    cddl
}

pub fn read_expect(path: &str) -> String {
    let path = PathBuf::from(std::env!("CARGO_MANIFEST_DIR"))
        .join("src/tests/expect")
        .join(path);
    let mut file = fs::File::open(path).unwrap();
    let mut expect = String::new();
    file.read_to_string(&mut expect).unwrap();
    expect
}
