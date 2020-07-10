# rendr

_A project scaffolding tool_

![MIT License](https://img.shields.io/github/license/jamf/rendr)
![GitHub release](https://img.shields.io/github/v/release/jamf/rendr)

Check out the [User Guide](https://jamf.github.io/rendr/) for the full documentation.

This project is currently under heavy development. The API is expected to change before reaching a 1.0.0 release. That said, it is functional and useful already! Try it out and provide any feedback you have by [opening an issue](https://github.com/jamf/rendr/issues).

## Features

_rendr_ is a scaffolding tool which allows generating entire projects (or anything else) from blueprints, using standard templating engines and simple customization via parameters. It is generic enough to apply to a wide variety of applications and tech stacks, but powerful and flexible enough to provide value, fast. The tool itself is really a generic template renderer. It's up to you, the template creator, to decide what to put in your template.

### Use cases

Here are just a few possible use cases:

* Enable rapid spin-up of new projects, complete with CI/CD pipelines, code quality gates, security analysis, and more
* Ship "Hello, World!" projects immediately to production, enabling instant iteration on features
* Include CI/CD standards baked into projects from the start, easily kept up to date
* Simplify repeated patterns like creating new microservices, libraries, or submodules on an existing project

## Installation

Install the CLI via [Homebrew](https://brew.sh):
```sh
brew install jamf/tap/rendr
```
Alternatively, download the CLI binary directly from the [Releases](https://github.com/jamf/rendr/releases) page and put it on your system path.

## Usage

More detailed usage can be found [in the User Guide](https://jamf.github.io/rendr/usage.html).

View available commands:
```sh
❯ rendr help
```

The basic usage to generate a project looks like this:
```sh
rendr init my-project --blueprint https://github.com/your/template
```

Provide values to the template with the `-v` flag:
```sh
rendr init my-project -b https://github.com/your/template -v name:foo -v version:1.0.0
```

## Contributing

Feedback and pull requests are welcome! Let us know if you have issues using the tool, or see use cases that are not yet supported. We'd love to expand its usefulness!
