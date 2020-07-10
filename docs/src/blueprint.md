# Creating blueprints

Blueprints consist of template files, scripts, and metadata.

* Template files live in the `template` directory, and are rendered by the templating engine
* Metadata is provided in a `metadata.yaml` file in the root of the blueprint directory. It lists specific values that can be provided to the template, among other things.
* Scripts live in a `scripts` directory in the blueprint directory. This is the place to customize the generated files or automate followup actions (like creating a remote repository or pipeline).

With these basic features, blueprints are already highly customizable! If you
have other use cases that are not supported, feel free to let us know in the
[issues](https://github.com/jamf/rendr/issues)!
