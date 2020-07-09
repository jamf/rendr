# rendr

_A project scaffolding tool_

![MIT License](https://img.shields.io/github/license/jamf/rendr)
![GitHub release](https://img.shields.io/github/v/release/jamf/rendr)

This project is currently under heavy development. The API is expected to change before reaching a 1.0.0 release. That said, it is functional and useful already! Try it out and provide any feedback you have by [opening an issue](https://github.com/jamf/rendr/issues).

## Features

_rendr_ is a scaffolding tool which allows generating entire projects (or anything else) from templates, using standard templating engines and simple customization via parameters. It is generic enough to apply to a wide variety of applications and tech stacks, but powerful and flexible enough to provide value, fast. The tool itself is really a generic template renderer. It's up to you, the template creator, to decide what to put in your template.

### Use cases

Here are just a few possible use cases:

* Enable rapid spin-up of new projects, complete with CI/CD pipelines, code quality gates, security analysis, and more
* Ship "Hello, World!" projects immediately to production, enabling instant iteration on features
* Include CI/CD standards baked into projects from the start, easily kept up to date
* Simplify repeated patterns like creating new microservices, libraries, or submodules on an existing project

### Template format

Templates consist of template files, scripts, and metadata. Full docs on the template format are coming soon!

* Template files live in the `blueprint` directory, and are rendered by the templating engine
* Metadata is provided in a `metadata.yaml` file in the root of the template repo. It lists specific values that can be provided to the template, among other things.
* Scripts live in a `scripts` directory in the template repo. This is the place to customize the generated files or automate followup actions (like creating a remote repository or pipeline).

With these basic features, templates are already highly customizable! If you have other use cases that are not supported, feel free to let us know in the [issues](https://github.com/jamf/rendr/issues)!

## Usage

Install the CLI via [Homebrew](https://brew.sh):
```sh
brew install jamf/tap/rendr
```
Alternatively, download the CLI binary directly from the [Releases](https://github.com/jamf/rendr/releases) page and put it on your system path.

View available commands:
```sh
❯ rendr help
A project scaffolding tool

USAGE:
    rendr [SUBCOMMAND]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    help    Prints this message or the help of the given subcommand
    init    Initializes a project from a template
```

The basic usage to generate a project looks like this:
```sh
rendr init my-project --template https://github.com/your/template
```

Provide values to the template with the `-v` flag:
```sh
rendr init my-project -t https://github.com/your/template -v name:foo -v version:1.0.0
```

## Development

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

## Contributing

Feedback and pull requests are welcome! Let us know if you have issues using the tool, or see use cases that are not yet supported. We'd love to expand its usefulness!
