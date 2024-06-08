import os
import sys
import subprocess

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
    exit_code = os.system("git push")
    if exit_code != 0:
        print("Failed to push changes, exiting")
        exit(1)

    print("Creating a new tag in git")
    exit_code = os.system(f"git tag {version}")
    if exit_code != 0:
        print("Failed to create a tag, exiting")
        exit(1)

    print("Pushing the tag to git")
    exit_code = os.system("git push --tags")
    if exit_code != 0:
        print("Failed to push the tag, exiting")
        exit(1)

    print("Publishing to crates.io")
    exit_code = os.system("cargo publish")
    if exit_code != 0:
        print("Failed to publish, exiting")
        exit(1)
    else:
        print("Published successfully")
else:
    print("Dry run, use --push to publish to crates.io")

    exit_code = os.system("cargo publish --dry-run")

    if exit_code != 0:
        print("Failed to do dry-run publish, exiting")
        exit(1)
    else:
        print("Dry-run succeeded, use --push to publish to crates.io")
