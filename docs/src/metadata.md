# Metadata

Each blueprint has a `metadata.yaml` file at its root level. This file defines
things like the blueprint name, version and description, as well as values
which the user can provide when rendering the template.

Here is a simple example:

```yaml
name: foo
version: 1
author: thecodesmith
description: A simple microservice blueprint
values:
- name: name
  description: The name of your project
  required: true
- name: port
  description: The port where the service listens
  default: "8000"
exclusions:
- "images/*"
```

Let's break it down:

Parameter     | Description
---------     | -----------
`name`        | The blueprint name
`version`     | The blueprint version
`author`      | The blueprint author
`description` | The blueprint description
`values`      | A list of values that will be provided to the template rendering
`exclusions`  | A list of glob patterns to exclude from rendering

The structure of each value in `values` looks like this:

Parameter     | Description
---------     | -----------
`name`        | The value key name
`description` | The description of this value; becomes the interactive prompt text
`required`    | Whether the value must be provided by the user (`true` or `false`)
`default`     | The default value if one is not provided by the user
