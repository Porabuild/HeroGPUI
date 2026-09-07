"""Gallery page sources. Component demos live in category modules."""

import os

COMPONENTS_DIR = os.path.join('gallery', 'src', 'pages', 'components')
DOCS = os.path.join('gallery', 'src', 'pages', 'docs.rs')


def component_sources():
    return tuple(sorted(
        os.path.join(COMPONENTS_DIR, name)
        for name in os.listdir(COMPONENTS_DIR)
        if name.endswith('.rs')
    ))


def page_sources():
    return component_sources() + (DOCS,)
