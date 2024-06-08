import os
import re
import subprocess


def get_version():
    # get the version from 'cargo pkgid' command
    pkgid = subprocess.run(['cargo', 'pkgid'], stdout=subprocess.PIPE)
    pkgid = pkgid.stdout.decode('utf-8')
    version = re.search(r'@(\d+\.\d+)\.\d+', pkgid).group(1)
    return version


def replace_version(readme, version):
    start = '<!--install instruction start-->'
    end = '<!--install instruction end-->'
    start_index = readme.find(start)
    end_index = readme.find(end)
    pkg = 'sparse_set_container'

    replacement = f'\n```toml\n[dependencies]\n{pkg} = "{version}"\n```\n'
    return readme[:start_index + len(start)] + replacement + readme[end_index:]


def replace_examples(readme):
    blocks = re.findall(r'<!--(.*)\.rs start-->', readme)
    print(f'Found examples: {blocks}')
    for block in blocks:
        start = f'<!--{block}.rs start-->'
        end = f'<!--{block}.rs end-->'
        start_index = readme.find(start)
        end_index = readme.find(end)

        if start_index == -1 or end_index == -1:
            print(f'Block {block} not found')
            continue

        with open(f'examples/{block}.rs', 'r') as file:
            code = file.read()

        code = code.strip()

        replacement = f'\n```rust\n{code}\n```\n'
        return readme[:start_index + len(start)] + replacement + readme[end_index:]


def update_benchmark_results(readme):
    benchmark_file = 'bench_table.md'

    if not os.path.exists(benchmark_file):
        print(f'Benchmark file {benchmark_file} not found, skipping benchmark results update')
        return readme

    print("Found benchmark results")

    start = '<!--benchmark table start-->'
    end = '<!--benchmark table end-->'

    start_index = readme.find(start)
    end_index = readme.find(end)

    if start_index == -1 or end_index == -1:
        print('Benchmark table not found')
        return readme

    with open(benchmark_file, 'r') as file:
        results = file.read()

    results = results.strip()

    return readme[:start_index + len(start)] + "\n" +results + "\n" + readme[end_index:]

version = get_version()
print(f'Latest version: {version}')

with open('README.md', 'r') as file:
    readme = file.read()

readme = replace_version(readme, version)

readme = replace_examples(readme)

readme = update_benchmark_results(readme)

with open('README.md', 'w') as file:
    file.write(readme)
