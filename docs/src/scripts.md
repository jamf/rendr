# Scripts

Blueprints can optionally contain scripts. This is a powerful mechanism that enables
blueprint creators to customize how the blueprint is rendered.

Scripts live in the `scripts` directory at the root level of the blueprint.

Currently only the `post-render.sh` script is supported. This can be expanded
in the future if there are other integration points where scripts are useful.

## The post-render script

If there is a script named `post-render.sh` in the blueprint's `scripts` directory, it will be
run automatically after the template files are rendered by the templating engine.

The values used to render the template are also provided to the script as
environment variables. So, if a user specifies `-v foo:42`, there will be a
variable named `foo` that can be accessed in the script like this:

```sh
#!/bin/sh

echo "The value of foo is $foo!"
```

This script will print:

    The value of foo is 42!

Use the post-render script to customize files and directories, or to create
remote repositories, CI/CD pipelines, etc.

**NOTE:** The script must be executable to be run. Make it executable like this:

    chmod +x scripts/post-render.sh
