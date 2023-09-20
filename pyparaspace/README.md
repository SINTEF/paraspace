`pyparaspace`: a Python wrapper for the [ParaSpace timelines planner](https://github.com/luteberget/paraspace) -- a simple, flexible and extensible solver for timeline-based planning problems using the [Z3 Theorem Prover](https://github.com/Z3Prover/z3).

# Installing

Install using Pip. 

```
pip install pyparaspace
```

See the file `testPyParaspace.py` for example usage.

# Building locally

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

# Building and releasing

This section is intended for package maintainers. The `pyparaspace`  package is
released on PyPi with Python wheel packages that make it convenient to use
`paraspace` without needing to set up Rust and C++ compilers and tools.
Through the `z3-sys` package's static link option, we get the whole planner,
including the Z3 solver, statically linked. This greatly increases the
convenience for users of the library.

Windows and Manylinux platforms are currently supported.


## Windows

If building and installing the local package works, then using `maturin build --release` 
should also correctly build a wheel package, which can be uploaded to PyPi using `maturin publish`.

## Manylinux

`paraspace` requires an Rust version 1.60 and Clang version 3.5 (to compile the Z3 solver), 
which makes it requires a bit of setup to correcly build the manylinux wheel. 
There is a Dockerfile available that can be used to build a Docker image with 
an up-to-date Rust version and version 7 of the LLVM/Clang toolchain.

The builds should work using the following commands.
```
docker build -t mybuild .
docker run --rm -v $(pwd):/io mybuild publish --skip-existing --compatibility manylinux2014 -i python3.10
```

