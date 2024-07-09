# -- Project information -----------------------------------------------------

project = 'Media Gateway'
copyright = '2024, BWSoft Management, LLC'
author = 'BWSoft'

version = '0.1.0'
release = version

# -- General configuration ---------------------------------------------------

# Add any Sphinx extension module names here, as strings. They can be
# extensions coming with Sphinx (named 'sphinx.ext.*') or your custom
# ones.
extensions = [
    'sphinx.ext.autodoc',
    'sphinx.ext.autosummary',
    'sphinx.ext.todo',
    'sphinx_rtd_theme',
    'sphinx.ext.intersphinx',
]

templates_path = ['_templates']
source_suffix = '.rst'
master_doc = 'index'
exclude_patterns = []
html_static_path = ['_static']
html_theme = 'sphinx_rtd_theme'
html_theme_options = {
    'display_version': True,
}
html_css_files = [
    'css/custom.css',
]

autosummary_generate = True
autosummary_imported_members = True
