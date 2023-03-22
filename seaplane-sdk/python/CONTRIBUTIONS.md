# Contributing to `Seaplane Python SDK`

Contributions are welcome!


## Dependencies

Install dependencies

```
$ poetry install
```

To [activate](https://python-poetry.org/docs/basic-usage#activating-the-virtual-environment) the virtual environment that is automatically created by Poetry:

```
$ poetry shell
```

To deactivate the environment:

```
(seaplane) $ exit
```

To upgrade all dependencies to their latest versions:

```
$ poetry update
```

## Packaging

To package the project as both a source distribution and a wheel:

```
$ poetry build
```

This will generate `dist/fact-1.0.0.tar.gz` and `dist/fact-1.0.0-py3-none-any.whl`.

Source and wheel redistributable packages can be [published to PyPI](https://python-poetry.org/docs/cli#publish) or installed directly from the filesystem using pip.

```
$ poetry publish
```

## Enforcing Code Quality

Automated code quality checks are performed using [Nox](https://nox.thea.codes/en/stable/) and [nox-poetry](https://nox-poetry.readthedocs.io/en/stable/)

To run all default sessions:

> Note: nox is installed into the virtual environment automatically by the poetry install command above. Run poetry shell to activate the virtual environment.

```
(seaplane) $ nox
```

## Testing

To pass arguments to pytest through nox:

```
(seaplane) $ nox -s test -- -k invalid_factorial
```

To run end to end tests you have to set the E2E_API_KEY env var:

```
(seaplane) $ export E2E_API_KEY="sp-your_api_key"
(seaplane) $ nox -s e2e
```

## Code Style Checking

[PEP 8](https://peps.python.org/pep-0008/) is the universally accepted style guide for
Python code. PEP 8 code compliance is verified using [Flake8](http://flake8.pycqa.org/). Flake8 is
configured in the `[tool.flake8]` section of `pyproject.toml`. Extra Flake8 plugins are also
included:

- `flake8-bugbear`: Find likely bugs and design problems in your program.
- `flake8-broken-line`: Forbid using backslashes (`\`) for line breaks.
- `flake8-comprehensions`: Helps write better `list`/`set`/`dict` comprehensions.
- `pep8-naming`: Ensure functions, classes, and variables are named with correct casing.
- `pyproject-flake8`: Allow configuration of `flake8` through `pyproject.toml`.

To lint code, run:

```bash
(seaplane) $ nox -s lint
```

## Automated Code Formatting

Code is automatically formatted using [black](https://github.com/psf/black). Imports are
automatically sorted and grouped using [isort](https://github.com/PyCQA/isort/).

These tools are configured by:

- [`pyproject.toml`](./pyproject.toml)

To automatically format code, run:

```bash
(fact) $ nox -s fmt
```

To verify code has been formatted, such as in a CI job:

```bash
(fact) $ nox -s fmt_check
```

## Type Checking

[Type annotations](https://docs.python.org/3/library/typing.html) allows developers to include
optional static typing information to Python source code. This allows static analyzers such
as [mypy](http://mypy-lang.org/), [PyCharm](https://www.jetbrains.com/pycharm/),
or [Pyright](https://github.com/microsoft/pyright) to check that functions are used with the correct types before runtime.

mypy is configured in [`pyproject.toml`](./pyproject.toml). To type check code, run:

```bash
(fact) $ nox -s type_check
```
See also [awesome-python-typing](https://github.com/typeddjango/awesome-python-typing).

## Publish a new version


> ⚠️ Only approved contributors can push tags to trigger a new release.

You have to create a new tag, adding the version of your package.

The version in `pyproject.toml` has to be bigger than the published package.

The tag convention is `sdk-py-v*` being `*` the version of the SDK.

```
git tag -a sdk-py-v0.2 -m "Python SDK v0.2 Release"
```