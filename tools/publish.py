import os
import sys

# run update_readme.py in order to make sure we have no changes there
os.system("python tools/update_readme.py")

# check that git doesn't have any changes
status = os.popen("git status --porcelain").read()
if status:
    print("Git has changes, commit them first")
    exit(1)

version = os.popen("cargo pkgid").read().split("@")[1].strip()
print(f"Current version is '{version}'")

push = "--push" in sys.argv
if push:
    print("Pushing all changes to git")
    os.system("git push")

    print("Creating a new tag in git")
    os.system(f"git tag {version}")

    print("Pushing the tag to git")
    os.system("git push --tags")

    print("Publishing to crates.io")
    os.system("cargo publish")
else:
    print("Dry run, use --push to publish to crates.io")
    os.system("cargo publish --dry-run")
