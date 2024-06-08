import os
import sys

# run update_readme.py in order to make sure we have no changes there
os.system("python tools/update_readme.py")

# check that git doesn't have any changes
status = os.popen("git status --porcelain").read()
if status:
    print("Git has changes, commit them first")
    exit(1)

push = "--push" in sys.argv
if push:
    print("Publishing to crates.io")
    os.system("cargo publish")
else:
    print("Dry run, use --push to publish to crates.io")
    os.system("cargo publish --dry-run")
