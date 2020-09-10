# These imports are managed by the blueprint.
import foo
# User imports start here.
{{@ imports }}
{{@ / }}

def entrypoint():
{{@ content }}
  print("Hello {{ name }}!")
{{@ / }}

def __main__():
  entrypoint()
