# Integration of Paraspace with the Unified Planning Library

The aim of this project is to make the
[Paraspace](https://github.com/luteberget/paraspace) planning engine available
in the [unified_planning library](https://github.com/aiplan4eu/unified-planning) 
by the [AIPlan4EU project](https://www.aiplan4eu-project.eu/).  Paraspace is a
simple, flexible and extensible solver for timeline-based planning problems
using Z3 and a novel abstraction refinement algorithm.

## Installation

Installing from PyPi is recommended because pre-built packages of Paraspace's
Python integration are available for Windows and Linux. 

```
pip install unified-planning up-paraspace
```

## Usage

```
from unified_planning.shortcuts import *
import up_paraspace

problem = Problem('myproblem')
# specify the problem (e.g. fluents, initial state, actions, goal)
...

planner = OneshotPlanner(name="paraspace")
result = planner.solve(problem)
print(result)
```


## Building/publishing

This section is intended for package maintainers.

Up-paraspace is a pure Python package that does not need to build any native
code.  The `pyparaspace` package that it depends on has a more involved build
process. Building and publishing is done with `hatch`. Run:

```
hatch publish
```





