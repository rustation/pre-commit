extern crate toml;
extern crate rustc_serialize;

use std::{env, io, fs, path};
use std::io::prelude::*;
use std::os::unix::fs::PermissionsExt;

fn main() {
    copy_file().unwrap();
}

fn find_cargo_toml(p: &path::Path) -> io::Result<fs::File> {
    let cargo_toml = p.join("Cargo.toml");

    println!("looking for toml in {:?}", cargo_toml);

    if cargo_toml.exists() {
        return fs::File::open(cargo_toml);
    }

    let parent = p.parent();

    match parent {
        Some(ref p) => return find_cargo_toml(p),
        None => return Err(io::Error::new(io::ErrorKind::NotFound, "Cargo.toml not found")),
    }
}

fn copy_file() -> io::Result<()> {
    let cwd = env::current_dir()?;
    let mut f = find_cargo_toml(&cwd)?;
    let mut s = String::new();
    f.read_to_string(&mut s)?;
    let contents = build_script(s);

    let hooks_dir = cwd.join(".git")
        .join("hooks");

    println!("Hooks dir {:?}", hooks_dir);

    if !hooks_dir.exists() {
        return Ok(());
    }

    let pre_commit = hooks_dir.join("pre-commit");

    let mut f = fs::File::create(&pre_commit)?;

    if cfg!(target_family = "unix") {
        let metadata = f.metadata()?;
        let mut permissions = metadata.permissions();
        permissions.set_mode(0o777);
        fs::set_permissions(&pre_commit, permissions)?;
    }

    f.write_all(contents.as_bytes())
}

fn build_script(s: String) -> String {
    let t = toml::Parser::new(&s).parse();

    let checks = t.as_ref()
        .and_then(|x| get_as_table("package", x))
        .and_then(|x| get_as_table("metadata", x))
        .and_then(|x| get_as_table("precommit", x))
        .iter()
        .flat_map(|xs| xs.iter())
        .map(format_test)
        .collect::<Vec<_>>()
        .join("\n");

    format_script(checks)
}

fn get_as_table<'a>(name: &str, x: &'a toml::Table) -> Option<&'a toml::Table> {
    x.get(name)
        .and_then(toml::Value::as_table)
}

fn format_script(s: String) -> String {
    format!(r#"
#!/bin/bash
set -eu

check_char='\xE2\x9C\x93'
cross_char='\xE2\x9C\x96'
green='\033[0;32m'
red='\033[0;31m'
nc='\033[0m'
check="$green$check_char$nc"
cross="$red$cross_char$nc"

{}"#,
            s)
}

fn format_test((k, v): (&String, &toml::Value)) -> String {

    format!(r#"printf "{}"

if result=$({}); then
    echo " $check"
else
    echo " $cross"
    echo " $result"
    exit 1
fi
"#,
            k,
            v.as_str().unwrap())
}