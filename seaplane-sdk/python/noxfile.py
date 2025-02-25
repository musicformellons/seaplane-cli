from tempfile import NamedTemporaryFile

import nox
from nox_poetry import Session, session

nox.options.error_on_external_run = True
nox.options.reuse_existing_virtualenvs = True
nox.options.sessions = ["fmt_check", "lint", "type_check", "test"]


@session()
def test(s: Session) -> None:
    s.install(".", "pytest", "pytest-cov")
    s.run(
        "python",
        "-m",
        "pytest",
        "-W",
        "ignore",
        "--cov=fact",
        "--cov-report=html",
        "--cov-report=term",
        "tests",
        *s.posargs,
    )


@session()
def e2e(s: Session) -> None:
    s.install(".", "pytest", "pytest-cov")
    s.run(
        "python",
        "-m",
        "pytest",
        "--cov=fact",
        "--cov-report=html",
        "--cov-report=term",
        "tests/end_to_end/e2e_sql_api.py",
        *s.posargs,
    )


# For some sessions, set venv_backend="none" to simply execute scripts within the existing Poetry
# environment. This requires that nox is run within `poetry shell` or using `poetry run nox ...`.
@session(venv_backend="none")
def fmt(s: Session) -> None:
    s.run("isort", ".")
    s.run("black", ".")


@session(venv_backend="none")
def fmt_check(s: Session) -> None:
    s.run("isort", "--check", ".")
    s.run("black", "--check", ".")


@session(venv_backend="none")
def lint(s: Session) -> None:
    # Run pyproject-flake8 entrypoint to support reading configuration from pyproject.toml.
    s.run("pflake8")


@session(venv_backend="none")
def type_check(s: Session) -> None:
    s.run("mypy", "src", "tests", "noxfile.py")


# noxfile.py
@nox.session(venv_backend="none")
def docs(s: Session) -> None:
    """Build the documentation."""
    # s.run("poetry", "install", "--no-dev", external=True)
    # install_with_constraints(session, "sphinx", "sphinx-autodoc-typehints")
    s.run("sphinx-build", "docs", "docs/_build")


# Note: This reuse_venv does not yet have affect due to:
#   https://github.com/wntrblm/nox/issues/488
@session(reuse_venv=False)
def licenses(s: Session) -> None:
    # Install dependencies without installing the package itself:
    #   https://github.com/cjolowicz/nox-poetry/issues/680
    with NamedTemporaryFile() as requirements_file:
        s.run_always(
            "poetry",
            "export",
            "--without-hashes",
            f"--output={requirements_file.name}",
            external=True,
        )
        s.install("pip-licenses", "-r", requirements_file.name)
    s.run("pip-licenses", *s.posargs)
