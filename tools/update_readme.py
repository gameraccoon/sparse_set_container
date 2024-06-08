import os
import re
import subprocess


def get_version():
    # get the version from 'cargo pkgid' command
    pkgid = subprocess.run(['cargo', 'pkgid'], stdout=subprocess.PIPE)
    pkgid = pkgid.stdout.decode('utf-8')
    version = re.search(r'@(\d+\.\d+\.\d+)', pkgid).group(1)
    return version


def replace_badge_links(readme, full_version):
    crates_io_shield = f'[crates.io shield]: https://img.shields.io/crates/v/sparse_set_container?label=latest'
    crates_io_link = f'[crates.io link]: https://crates.io/crates/sparse_set_container'
    docs_rs_badge = f'[docs.rs badge]: https://docs.rs/sparse_set_container/badge.svg?version={version}'
    docs_rs_link = f'[docs.rs link]: https://docs.rs/sparse_set_container/{version}/sparse_set_container/'
    shields_io_download_count = f'[shields.io download count]: https://img.shields.io/crates/d/sparse_set_container.svg'

    badges = f'{crates_io_shield}\n{crates_io_link}\n{docs_rs_badge}\n{docs_rs_link}\n{shields_io_download_count}'

    start = '<!--badge links start-->'
    end = '<!--badge links end-->'
    start_index = readme.find(start)
    end_index = readme.find(end)

    return readme[:start_index + len(start)] + "\n" + badges + "\n" + readme[end_index:]


def replace_install_instruction(readme, version):
    start = '<!--install instruction start-->'
    end = '<!--install instruction end-->'
    start_index = readme.find(start)
    end_index = readme.find(end)
    pkg = 'sparse_set_container'

    short_version = '.'.join(version.split('.')[:2])

    replacement = f'\n```toml\n[dependencies]\n{pkg} = "{short_version}"\n```\n'
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

readme = replace_badge_links(readme, version)

readme = replace_install_instruction(readme, version)

readme = replace_examples(readme)

readme = update_benchmark_results(readme)

with open('README.md', 'w') as file:
    file.write(readme)
