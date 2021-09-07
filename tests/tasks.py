# type: ignore
from invoke import call, task

SOURCES = "tests.py"


@task
def black(c, check=False):
    print("Running black")
    cmd = f"black {SOURCES}"
    if check:
        cmd += " --check"
    c.run(cmd)


@task
def isort(c, check=False):
    print("Running isort")
    cmd = f"isort {SOURCES}"
    if check:
        cmd += " --check"
    c.run(cmd)


@task
def flake8(c):
    print("Running flake8")
    c.run(f"flake8 {SOURCES}")


@task(
    pre=[
        call(black, check=True),
        call(isort, check=True),
        call(flake8),
    ]
)
def lint(c):
    pass


@task
def safety_check(c):
    c.run("safety check")
