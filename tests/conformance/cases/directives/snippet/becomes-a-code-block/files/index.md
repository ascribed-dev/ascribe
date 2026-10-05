Whole file:

@snippet: code:app.py

A region, dedented, with tag lines and removed lines left out:

@snippet: code:app.py#main

With phrases, a title, and another language:

@snippet {lang=python, title="Run it", phrases=true}: code:app.py#main

A file with no extension has no language:

@snippet: code:install
