# Rendering Blueprints

## See the options

View the available commands with `rendr --help`:

    ❯ rendr --help
    A project scaffolding tool

    USAGE:
        rendr [SUBCOMMAND]

    FLAGS:
        -h, --help       Prints help information
        -V, --version    Prints version information

    SUBCOMMANDS:
        create              Creates a project from a blueprint
        create-blueprint    Creates a new blueprint scaffold
        help                Prints this message or the help of the given subcommand(s)
        info                Displays blueprint info for an existing project

To see usage for a subcommand, use `rendr [command] --help`:

    ❯ rendr create --help
    rendr-create
    Creates a project from a blueprint

    USAGE:
        rendr create [FLAGS] [OPTIONS] --blueprint <blueprint> --dir <dir>

    FLAGS:
            --debug          Enables debug logging
            --git-init       Initializes a Git repository in the rendered project
        -h, --help           Prints help information
            --no-git-init    Skips initializing Git repository in the rendered project
        -V, --version        Prints version information
        -w, --watch          After generating the project, watch the blueprint files for changes and regenerate the project
                             on change

    OPTIONS:
        -b, --blueprint <blueprint>    The location of the blueprint (a Git repo or a local directory)
        -d, --dir <dir>                The output directory name
        -n, --name <name>              The name of the project
        -p, --password <password>      The password for Git authentication (insecure - use the GIT_PASS env var instead)
        -k, --ssh-key <ssh-key>        The path to the private SSH key for Git auth
        -u, --user <user>              The user for Git authentication
        -v, --value <value>...         Custom value provided the blueprint (flag may be repeated)

## Render a project

Use `rendr create` to render a project from a blueprint. The basic usage looks
like this:

```sh
rendr create --blueprint https://github.com/your/template --dir my-project
```

## Provide custom values

By default, if you don't provide any values when running `rendr create`, you will
be prompted for each required value. To provide values non-interactively, use
the `-v` flag. This flag is repeated once for each value.

```sh
rendr create -b https://github.com/your/template -d my-project -v name:foo -v version:1.0.0
```

## Important! A note about scripts

Blueprints can contain scripts that execute as part of the rendering process.
Blueprint creators can use this mechanism to provide custom functionality when
the blueprint is rendered, like creating a repository or configuring a CI
pipeline. However, be careful to only use templates from trusted sources, as
these scripts execute with the same privileges as the user that invoked them.
A malicious template script could modify files on your system, send your
personal data somewhere, install malware, etc.

## Upgrading to a new blueprint version

If a new version is released of the blueprint used in your project, your
project can be easily upgraded using the `rendr upgrade` command. Blueprint
authors can include new files, or make custom upgrades to existing files in a
project. The upgrade logic is entirely up to the blueprint author.

This is a powerful mechanism that allows maintaining common code, configuration
and best practices across multiple codebases.

Use `rendr upgrade --help` for more details on usage.

## Blueprint developer mode

If you are developing a blueprint, you will likely want to edit your templates
and then render them to see the output, make more edits, render them again, in
a continuous loop.  To make this easier, use the `--watch` (`-w`) flag for
`rendr create`.

```sh
rendr create --blueprint ../my-blueprint --dir foo --watch
Output directory: "foo". Creating your new scaffold...
Success. Enjoy!
Watching for blueprint changes...
```

The next time you make changes to the files in the `my-blueprint` directory,
the entire blueprint will be re-rendered automatically. Use `Ctrl-C` to cancel
the watch command.
