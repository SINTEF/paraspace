# Integration of ParaSpace with the Unified Planning Library

The aim of this project is to make the
[ParaSpace](https://github.com/luteberget/paraspace) planning engine available
in the [unified_planning library](https://github.com/aiplan4eu/unified-planning) 
by the [AIPlan4EU project](https://www.aiplan4eu-project.eu/).  ParaSpace is a
simple, flexible and extensible solver for timeline-based planning problems
using Z3 and a novel abstraction refinement algorithm.

## Installation

Installing from PyPi is recommended because pre-built packages of ParaSpace's
Python integration are available for Windows and Linux. 

## Usage

```
from unified_planning.shortcuts import *
from unified_planning.engines import PlanGenerationResultStatus

problem = Problem('myproblem')
# specify the problem (e.g. fluents, initial state, actions, goal)
...

planner = OneshotPlanner(name="paraspace")
result = planner.solve(problem)
if result.status == PlanGenerationResultStatus.SOLVED_SATISFICING:
    print(f'{Found a plan.\nThe plan is: {result.plan}')
else:
    print("No plan found.")
```






