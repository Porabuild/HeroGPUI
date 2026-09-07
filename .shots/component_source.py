"""Read a Rust module that is either `foo.rs` or a `foo/` directory.

Parity audits and extractors keep the historical name (`color_picker.rs`,
`date_picker.rs`, `reference_metadata.rs`). After a split the name still
resolves: a missing `foo.rs` whose `foo/` directory exists is the
concatenation of every `.rs` file in that tree, in sorted path order.
A path that exists as a file is read as itself, so CSS and other
non-module inputs keep working.
"""

from __future__ import annotations

import io
import os

SRC = os.path.join('crates', 'herogpui-components', 'src')


def read_path(path, encoding='utf-8', errors='strict'):
    """Read `path`, or concatenate `path[:-3]/**/*.rs` when `path` is a split module."""
    if os.path.isfile(path):
        return io.open(path, encoding=encoding, errors=errors).read()
    if path.endswith('.rs'):
        directory = path[:-3]
        if os.path.isdir(directory):
            files = []
            for root, dirs, names in os.walk(directory):
                dirs.sort()
                for name in sorted(names):
                    if name.endswith('.rs'):
                        files.append(os.path.join(root, name))
            if not files:
                raise FileNotFoundError(path)
            return ''.join(
                io.open(name, encoding=encoding, errors=errors).read()
                for name in files
            )
    raise FileNotFoundError(path)


def module_exists(path):
    return os.path.isfile(path) or (path.endswith('.rs') and os.path.isdir(path[:-3]))


def list_modules(src_dir=SRC, skip=('lib.rs',)):
    """Logical module names (`color_picker.rs`) for files and directory modules."""
    names = []
    for name in os.listdir(src_dir):
        if name in skip:
            continue
        path = os.path.join(src_dir, name)
        if name.endswith('.rs') and os.path.isfile(path):
            names.append(name)
        elif os.path.isdir(path) and not name.startswith('.'):
            names.append(name + '.rs')
    names.sort()
    return names


def read_module(name, src_dir=SRC, encoding='utf-8', errors='strict'):
    return read_path(os.path.join(src_dir, name), encoding=encoding, errors=errors)
