# Local iteration

You'll need Rust and Cargo. [You can get the whole toolchain here!](https://rustup.rs/)

After checking out this repository, `cd` into the directory. You can use the standard Cargo commands. Below you'll find the most important ones.

### Compile the whole project

This should get you the usual dev build.
```sh
cargo build
```

This one is the same, but it produces an optimized release build. It takes more time.
```sh
cargo build --release
```

### Tests

Run the unit tests and documentation tests.
```sh
cargo test
```

### Docs

Build the reference manual and open it in your browser.
```sh
cargo doc --open
```

### Run the application

Run the application.
```sh
cargo run
```

Get the helpfile.
```sh
cargo run -- -h
```

Get the help text for the `create` subcommand.
```sh
cargo run -- create -h
```

Initialize a project from the [Go microservice blueprint](https://github.com/jamf/rendr-sample-blueprint-go-microservice).
```sh
cargo run -- create --blueprint https://github.com/jamf/rendr-sample-blueprint-go-microservice.git --dir my-project -v name:foo
```
