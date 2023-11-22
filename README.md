# ParaSpace timelines planner

Paraspace is a simple, flexible, and extensible planner software for solving timeline-based planning problems using [Z3 Theorem Prover](https://github.com/Z3Prover/z3). 
The software is available as a standalone software package or as part of the [unified_planning library](https://github.com/aiplan4eu/unified-planning) 
The methodology used to develop the planner is described in this [paper](https://www.sciencedirect.com/science/article/pii/S2405896322024764).  

# Installation

`pyparaspace` is a Python wrapper of the paraspace for easier usage for users and it's recommended to install using Pip. 

```
pip install pyparaspace
```

## Building locally

Requirements: Rust, Cargo, Clang/LLVM/LibClang, CMake.

 * Create a virtual environment
```
python3 -m venv env
source env/bin/activate
```

 * Install maturin
```
pip install maturin
```

 * Build package
```
maturin develop
```

## Building and releasing

This section is intended for package maintainers. The `pyparaspace`  package is
released on PyPi with Python wheel packages that make it convenient to use
`paraspace` without needing to set up Rust and C++ compilers and tools.
Through the `z3-sys` package's static link option, we get the whole planner,
including the Z3 solver, statically linked. This greatly increases the
convenience for users of the library.

Windows and Manylinux platforms are currently supported.


### Windows

If building and installing the local package works, then using `maturin build --release` 
should also correctly build a wheel package, which can be uploaded to PyPi using `maturin publish`.

### Manylinux

`paraspace` requires an Rust version 1.60 and Clang version 3.5 (to compile the Z3 solver), 
which makes it require a bit of setup to correctly build the manylinux wheel. 
There is a Dockerfile available that can be used to build a Docker image with 
an up-to-date Rust version and version 7 of the LLVM/Clang toolchain.

The builds should work using the following commands.
```
docker build -t mybuild .
docker run --rm -v $(pwd):/io mybuild publish --skip-existing --compatibility manylinux2014 -i python3.10
```

# Example Usage
Below is an example of the planner used to solve the problem of a robot moving between two locations locA and locB. 

```
import pyparaspace as pps

locA = pps.TokenType(value="locA",conditions=[],duration_limits=(1,None),capacity=0)
moveAtoB = pps.TokenType(value="moveAtoB",conditions=
                         [pps.TemporalCond(temporal_relation=pps.TemporalRelation.MetBy,amount=0,timeline="location",value="locA"),
                          pps.TemporalCond(temporal_relation=pps.TemporalRelation.Meets,amount=0,timeline="location",value="locB")
                          ],duration_limits=(2,3),capacity=0)
locB = pps.TokenType(value="locB",conditions=
                     [pps.TemporalCond(temporal_relation=pps.TemporalRelation.MetBy,amount=0,timeline="location",value="moveAtoB")
                      ],duration_limits=(1,None),capacity=0)

init = pps.StaticToken(value="locA",const_time=pps.fact(0),capacity=0,conditions=[])
goal = pps.StaticToken(value="locB",const_time=pps.goal(),capacity=0,conditions=[])

location = pps.Timeline(name="location",token_types=[locA,moveAtoB,locB],static_tokens=[init,goal])


solution = pps.solve(pps.Problem(timelines=[location]))

```

See the file `testPyParaspace.py` for more examples.

# Integration of ParaSpace with the Unified Planning Library

The software is also available as a part of the [unified_planning library](https://github.com/aiplan4eu/unified-planning) developed
by the [AIPlan4EU project](https://www.aiplan4eu-project.eu/).

## Installation

Installing from PyPi is recommended because pre-built packages of ParaSpace's
Python integration is available for Windows and Linux. 

```
pip install unified-planning up-paraspace
```

## Example Usage
Below is an example of how to use the paraspace planner through the unified planning framework. 
Documentation of UPF's features and usage is available [here](https://unified-planning.readthedocs.io/en/latest/)
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

# Short API Documentation
The short API documentation of the most essential data types and functions of the paraspace software is described in the [DOCUMENTATION](docs/DOCUMENTATION.md) file.

# Licence
This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

# Contributing
We welcome contributions! Please read our [Contribution Guidelines](docs/CONTRIBUTING.md) for details on how to get started.

# Support and Contact
If you have any questions or need assistance, please contact us at bjornar.luteberget@sintef.no or synne.fossoy@sintef.no

# Acknowledgments
The paraspace library has been developed as part of the [ROBPLAN](https://www.sintef.no/en/projects/2021/robplan/) project funded by the Norwegian Research Council (RCN), grant number 322744.
The UPF-integration of paraspace has been developed for the [AIPLAN4EU](https://aiplan4eu-project.eu) H2020 project, grant number 101016442.
