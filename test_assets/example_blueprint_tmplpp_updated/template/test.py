# These imports are managed by the blueprint.
import foo
import bar
# User imports start here.
{{@ imports }}
{{@ / }}

def entrypoint(something):
{{@ content }}
  print("Hello {{ name }}!")
{{@ / }}

def __main__():
  entrypoint()
