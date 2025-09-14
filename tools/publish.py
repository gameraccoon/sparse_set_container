import os
import sys
import subprocess

# run update_readme.py in order to make sure we have no changes there
os.system("python tools/update_readme.py")

# check that git doesn't have any changes
status = os.popen("git status --porcelain").read()
if status:
    print("\nGit has changes, commit them first")
    exit(1)

# get the version after @ or # from the result of `cargo pkgid`
pkgid_output = os.popen("cargo pkgid").read()
pkgid_output_split = pkgid_output.split("@")
if len(pkgid_output_split) == 1:
    pkgid_output_split = pkgid_output.split("#")
version = pkgid_output_split[1].strip()
print(f"Current version is '{version}'")

# check that git doesn't have the tag locally
if subprocess.run(["git", "tag", "-l", f"v{version}"], capture_output=True).stdout.decode("utf-8").strip() != "":
    print(f"\nTag v{version} already exists in the git repo locally, exiting")
    exit(1)

# check that git doesn't have the tag remotely
if subprocess.run(["git", "ls-remote", "--tags", "origin", f"refs/tags/v{version}"], capture_output=True).stdout.decode("utf-8").strip() != "":
    print(f"\nTag v{version} already exists in the git repo on the remote, exiting")
    exit(1)

# check that examples can be compiled
if subprocess.run(["cargo", "build", "--examples", "--quiet"], capture_output=True).returncode != 0:
    os.system("cargo build --examples --quiet")
    print("\nExamples failed to compile, exiting")
    exit(1)
print("Readme examples compiled successfully")

# check that tests are passing (suppress test output)
if subprocess.run(["cargo", "test", "--quiet"], capture_output=True).returncode != 0:
    os.system("cargo test --quiet --no-fail-fast")
    print("\nTests failed, exiting")
    exit(1)
print("Tests passed")

push = "--push" in sys.argv
if push:
    print("Pushing all changes to git")
    exit_code = os.system("git push")
    if exit_code != 0:
        print("\nFailed to push changes, exiting")
        exit(1)

    print("Creating a new tag in git")
    exit_code = os.system(f"git tag v{version}")
    if exit_code != 0:
        print("\nFailed to create a tag, exiting")
        exit(1)

    print("Pushing the tag to git")
    exit_code = os.system("git push --tags")
    if exit_code != 0:
        print("\nFailed to push the tag, exiting")
        exit(1)

    print("Publishing to crates.io")
    exit_code = os.system("cargo publish")
    if exit_code != 0:
        print("\nFailed to publish, exiting")
        exit(1)
    else:
        print("Published successfully")
else:
    print("Dry run, use --push to publish to crates.io")

    exit_code = os.system("cargo publish --dry-run")

    if exit_code != 0:
        print("\nFailed to do dry-run publish, exiting")
        exit(1)
    else:
        print("Dry-run succeeded, use --push to publish to crates.io")
