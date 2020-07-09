# Usage

Install the CLI via [Homebrew](https://brew.sh):
```sh
brew install jamf/tap/express
```
Alternatively, download the CLI binary directly from the [Releases](https://github.com/jamf/express/releases) page and put it on your system path.

View available commands:
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

The basic usage to generate a project looks like this:
```sh
express init my-project --template https://github.com/your/template
```

Provide values to the template with the `-v` flag:
```sh
express init my-project -t https://github.com/your/template -v name:foo -v version:1.0.0
```