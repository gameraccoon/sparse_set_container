import os
import re
import subprocess

benchmark_output = subprocess.run(['cargo', 'run', '--example', 'bench', '--release'], stdout=subprocess.PIPE)
benchmark_output = benchmark_output.stdout.decode('utf-8')

benchmark_groups = [
    {
        'name': 'SparseSet<String>',
        'suffix': '_sparse_set'
    },
    {
        'name': 'Vec<String>',
        'suffix': '_vec'
    },
    {
        'name': 'HashMap<i32, String>',
        'suffix': '_hash_map'
    },
    {
        'name': 'thunderdome::Arena<String>',
        'suffix': '_thunderdome_arena'
    },
    {
        'name': 'generational_arena::Arena<String>',
        'suffix': '_generational_arena'
    },
    {
        'name': 'slotmap::SlotMap<slotmap::DefaultKey, String>',
        'suffix': '_slot_map'
    },
    {
        'name': 'slab::Slab<String>',
        'suffix': '_slab'
    }
]

benchmarks = [
    {
        'name': 'Create empty',
        'benchmark_name': 'create_empty',
    },
    {
        'name': 'Create with capacity',
        'benchmark_name': 'create_with_capacity',
    },
    {
        'name': 'Push 100 elements',
        'benchmark_name': 'push_hundred_elements',
    },
    {
        'name': 'With capacity push 100',
        'benchmark_name': 'create_with_capacity_and_push_hundred_elements',
    },
    {
        'name': 'Lookup 100 elements',
        'benchmark_name': 'get_hundred_elements',
    },
    {
        'name': 'Iterate over 100 elements',
        'benchmark_name': 'iterate_over_hundred_elements',
    },
    {
        'name': 'Clone with 100 elements',
        'benchmark_name': 'clone_with_hundred_elements',
    },
    {
        'name': 'Clone 100 and remove 10',
        'benchmark_name': 'clone_and_remove_ten_out_of_hundred_elements',
    },
    {
        'name': 'Clone 100 and swap_remove 10',
        'benchmark_name': 'clone_and_swap_remove_ten_out_of_hundred_elements',
    },
]

def parse_benchmark_line(line):
    parts = line.split()
    name = parts[1]
    time = parts[4]
    error = parts[7].strip(')')
    return name, time, error


def parse_benchmark_output(output):
    lines = output.split('\n')
    # ignore lines that don't end with ')'
    lines = [line for line in lines if line.endswith(')')]
    # get dict of benchmark results
    benchmarks = {}
    for line in lines:
        name, time, error = parse_benchmark_line(line)
        benchmarks[name] = (time, error)
    return benchmarks


print("Started collecting benchmarks...")

benchmark_results = parse_benchmark_output(benchmark_output)

print("Benchmarks collected")

# make a table for printing
table = []
# header
table.append(['Benchmark'] + ["`" + group['name'] + "`" for group in benchmark_groups])
for benchmark in benchmarks:
    row = [benchmark['name']]
    bench_name = benchmark['benchmark_name']
    for group in benchmark_groups:
        suffix = group['suffix']
        key = bench_name + suffix
        if key in benchmark_results:
            time, error = benchmark_results[key]
            row.append(f'{time} ns ±{error}')
        else:
            row.append('N/A')

    table.append(row)

# print a markdown table
table_str = ''
for row in table:
    table_str += '| ' + ' | '.join(row) + ' |\n'
    if row == table[0]:
        table_str += '| ' + ' | '.join(['---' for _ in row]) + '|\n'

bench_file_name = 'bench_table.md'
with open(bench_file_name, 'w') as f:
    f.write(table_str)

print(f"New data has been written to {bench_file_name}")
