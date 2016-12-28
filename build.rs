extern crate toml;
extern crate rustc_serialize;

use std::{env, io, fs, path};
use std::io::prelude::*;

fn main() {
    copy_file().unwrap();
}

fn copy_file() -> io::Result<()> {
    let dir = env::var("PWD").unwrap();
    let cwd = path::Path::new(&dir);

    let cargo_toml = cwd.clone()
        .join("cargo.toml");
    let mut f = fs::File::open(cargo_toml)?;
    let mut s = String::new();
    f.read_to_string(&mut s)?;
    let contents = build_script(s);

    let pre_commit = cwd.join(".git")
        .join("hooks")
        .join("pre-commit");

    println!("{:?}", pre_commit);

    let mut f = fs::File::create(&pre_commit)?;
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
errors=0

{}

if [ "$errors" != 0 ]; then
	echo "Failed"
	exit 1
else
	echo "OK"
fi"#,
            s)
}

fn format_test((k, v): (&String, &toml::Value)) -> String {

    format!(r#"echo -n {}

if result=$({}); then
    echo -e " $check"
else
    echo -e " $cross"
    echo -e " $result"
    errors=1
fi
"#,
            k,
            v)
}