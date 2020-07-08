# express

_A project scaffolding tool_

![MIT License](https://img.shields.io/github/license/jamf/express)
![GitHub release](https://img.shields.io/github/v/release/jamf/express)

This project is currently under heavy development. The API is expected to change before reaching a 1.0.0 release. That said, it is functional and useful already! Try it out and provide any feedback you have by [opening an issue](https://github.com/jamf/express/issues).

## Features

_Express_ is a scaffolding tool which allows generating entire projects (or anything else) from templates, using standard templating engines and simple customization via parameters. It is generic enough to apply to a wide variety of applications and tech stacks, but powerful and flexible enough to provide value, fast. The tool itself is really a generic template renderer. It's up to you, the template creator, to decide what to put in your template.

Here are just a few possible use cases:

* Enable rapid spin-up of new projects, complete with CI/CD pipelines, code quality gates, security analysis, and more
* Ship "Hello, World!" projects immediately to production, enabling instant iteration on features
* CI/CD standards baked into projects from the start, easily kept up to date
* Simplify repeated patterns like creating new microservices, libraries, or submodules on an existing project
* Foster innovation by enabling rapid prototyping of new applications

The basic usage looks like this:
```sh
express init my-project --template https://github.com/your/blueprint
```

Blueprints consist of template files, scripts, and metadata.

* Template files are simply text files that are rendered by a templating engine. Currently, [Mustache](http://mustache.github.io) is the embedded templating engine. This may expand in the future.
* Metadata is provided in a `metadata.yaml` file in the root of the blueprint repo. It lists specific values that can be provided to the template. The user will be prompted for these values when generating the project. Values can also be provided with the `-v` flag for non-interactive use.
* Scripts live in a `scripts` directory in the blueprint repo. Currently supported is the `post-render.sh` script, which runs after the templates are rendered. This is the place to customize the generated files or provide followup actions (like creating a remote repository or pipeline).

With these basic features, templates are already highly customizable! If you have other use cases that are not supported, feel free to let us know in the [issues](https://github.com/jamf/express/issues)!

Full docs coming soon.

## Usage

Install the CLI via Homebrew, or downloading directly from the [Releases](https://github.com/jamf/express/releases) page.
```sh
brew install jamf/tap/express
```

View avaiable commands:
```sh
❯ express help
A project scaffolding tool

USAGE:
    express [SUBCOMMAND]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    help    Prints this message or the help of the given subcommand
    init    Initializes a project from a template
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

Initialize a project from the [example-blueprint](https://stash.jamf.build/projects/SCAF/repos/example-blueprint/browse) blueprint.
```sh
cargo run -- init --template https://stash.jamf.build/scm/scaf/example-blueprint.git my-project -v name:foo
```

## Contributing

Feedback and pull requests are welcome!
