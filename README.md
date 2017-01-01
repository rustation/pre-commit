# pre-commit

Reads hooks metadata from Cargo.toml and executes on commit



### Installing

```
[dependencies]
pre-commit = "0.5.2"
```

### Usage

Add a table like the following to your `Cargo.toml`

```
[package.metadata.precommit]
fmt = "cargo fmt -- --write-mode diff 2>&1"
test = "cargo test 2>&1"
```

Then run:

```cargo clean; cargo build;```

You should now have a `pre-commit` file in your `./git/hooks` that will run the listed pre-commit entries.
