# Template

Each blueprint has a `template` directory at its root level. This directory
contains all the template files and directories that will be rendered into the
final project.

## Template engine

The template engine used by `rendr` is [Mustache](http://mustache.github.io),
"The logic-less template engine". See the link for the (extremely simple)
Mustache manual and demos to get started.

To get started even without reading the manual, it's enough to know that
Mustache uses "tags", which are indicated by double "mustaches", like `{{ name
}}` or `{{ my_custom_url }}`. When given a context of key/value pairs, Mustache
replaces the tags with the values from the context. For example, given this
template:

	Hello {{ name }}!

and this context:

	name:foo

will produce the following text:

	Hello foo!

Pretty simple.

`rendr` handles passing the context to the templating engine for you. The way
to provide values to the rendering context is by using `rendr`'s "values",
which are defined in the `metadata.yaml` and provided by the user via prompts
or the `--value` flag when rendering the blueprint. See the [metadata
format](metadata.md) and the [command line usage](usage.md) for more details
there.

## Sample `template` directory

Here's a concrete example of creating templates in `rendr`. We have a `template`
directory with just two files:

	template
	├── main.go
	└── README.md

The `README.md` has this for contents:

	# Project {{ name }}

	Welcome to your new project!

	Please run `go run main.go` to run the app at http://localhost:{{ port }}.

You will notice that the template has two tags, `name` and `port`. These must
be defined in the `metadata.yaml` file like this:

```yaml
...
values:
- name: name
  description: The name of the project
  required: true
- name: port
  description: The port where the app runs
  default: "8000"
```

These values are then provided by the user rendering the blueprint like this:

	rendr create --dir foo --blueprint <url> --value name:foo --value port:3000

This will result in all the files in the `template` directory being rendered by
the template engine and copied into the specified new project directory (in
this case named `foo`):

	foo
	├── main.go
	└── README.md

The contents of the rendered `README.md` look like this:

	# Project foo

	Welcome to your new project!

	Please run `go run main.go` to run the app at http://localhost:3000.

Any files or directories you like can go in the `template` directory, and they
will rendered into the generated project directory.

## Excluding files

Sometimes your template will contain files that you don't want to render with
Mustache, and want them to be copied over to the rendered project without
modification. Some examples of this would be binary files like images, or
third-party files that are included in the project. These files can be excluded
by using the `exclusions:` list in `metadata.yaml`. See the
[Metadata](metadata.md) docs for details.

## Dynamic file or directory names

Sometimes you want your rendered files or directories to have custom names
based on the values the user supplied. This can't be done using the Mustache
templating engine directly. The way to accomplish this is to rename the files
or directories in the `post-render.sh` script. See [Scripts](scripts.md) for
more details.

## Additional use cases

Between the Mustache templating and customization via pre- and post-render
scripts, `rendr` provides immense flexibility to generate projects for nearly
any tech stack. If you have specific use cases that you find are not supported,
please open a issue over at the [GitHub
Issues](https://github.com/jamf/rendr/issues), describe your problem and what
functionality is missing, and we will consider it.

## Support

Still need help or more examples? Open an issue at `rendr`'s [GitHub
Issues](https://github.com/jamf/rendr/issues) and describe your problem, and
we'd be happy to help!
