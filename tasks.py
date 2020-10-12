import os
from invoke import task

@task
def newday(c, day):

    day = int(day)

    c.run(f"mkdir -p puzzles/{day:d}/")

    if not os.path.exists(f"src/puzzles/day{day:d}.rs"):
        c.run(f"cp templates/dayn.rs src/puzzles/day{day:d}.rs")