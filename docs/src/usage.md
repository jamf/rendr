# Rendering Blueprints

## See the options

View the available commands with `rendr --help`:

    ❯ rendr --help
    rendr 0.3.1-alpha.0
    A project scaffolding tool

    USAGE:
        rendr [SUBCOMMAND]

    FLAGS:
        -h, --help       Prints help information
        -V, --version    Prints version information

    SUBCOMMANDS:
        help    Prints this message or the help of the given subcommand
        init    Initializes a project from a blueprint

To see usage for a subcommand, use `rendr [command] --help`:

    ❯ rendr init --help
    Initializes a project from a blueprint

    USAGE:
        rendr init [OPTIONS] <name> --template <template>

    FLAGS:
        -h, --help       Prints help information
        -V, --version    Prints version information

    OPTIONS:
        -d, --dir <dir>              The output directory name
        -t, --template <template>    The URI of the template repo
        -v, --value <value>...       Value(s) to render the blueprint with

    ARGS:
        <name>    The name of the project

## Render a project

Use `rendr init` to render a project from a blueprint. The basic usage looks like this:

```sh
rendr init my-project --template https://github.com/your/template
```

## Provide custom values

By default, if you don't provide any values when running `rendr init`, you will
be prompted for each required value. To provide values non-interactively, use
the `-v` flag. This flag is repeated once for each value.

```sh
rendr init foo -t https://github.com/your/template -v name:foo -v version:1.0.0
```

## Important! A note about scripts

Blueprints can contain scripts that execute as part of the rendering process.
Blueprint creators can use this mechanism to provide custom functionality when
the blueprint is rendered, like creating a repository or configuring a CI
pipeline. However, be careful to only use templates from trusted sources, as
these scripts execute with the same privileges as the user that invoked them.
A malicious template script could modify files on your system, send your
personal data somewhere, install malware, etc.
