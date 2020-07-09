# Creating blueprints

Blueprints consist of template files, scripts, and metadata.

* Template files live in the `blueprint` directory, and are rendered by the templating engine
* Metadata is provided in a `metadata.yaml` file in the root of the template repo. It lists specific values that can be provided to the template, among other things.
* Scripts live in a `scripts` directory in the template repo. This is the place to customize the generated files or automate followup actions (like creating a remote repository or pipeline).

With these basic features, templates are already highly customizable! If you
have other use cases that are not supported, feel free to let us know in the
[issues](https://github.com/jamf/rendr/issues)!
