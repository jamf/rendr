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

Get the helpfile for the `init` subcommand.
```sh
cargo run -- init -h
```

Initialize a project from the [Go microservice blueprint](https://stash.jamf.build/projects/SCAF/repos/blueprint-go-microservice/browse).
```sh
cargo run -- init --template https://stash.jamf.build/scm/scaf/blueprint-go-microservice.git my-project -v name:foo
```