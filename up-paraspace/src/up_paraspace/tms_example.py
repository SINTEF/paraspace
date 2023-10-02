from typing import Tuple
from unified_planning.shortcuts import *
import time

def define_finset_numbered(problem: Problem, type_name: str, n: int) -> Tuple[UserType, List[Object]]:
    return define_finset_named(problem, type_name, [f"{type_name.lower()}{i+1}" for i in range(n)])


def define_finset_named(problem: Problem, type_name: str, names: List[str]) -> Tuple[UserType, List[Object]]:
    type_ = UserType(type_name)
    objs = [problem.add_object(name, type_) for name in names]
    return type_, objs


def mk_tms_timeline_simple(n_kilns: int, n_pieces: int):
    problem = Problem("tms")

    Kiln, kilns = define_finset_numbered(problem, "Kiln", n_kilns)
    Piece, pieces = define_finset_numbered(problem, "Piece", n_pieces)
    KilnStateType, [KilnReady, KilnFire] = define_finset_named(problem, "KilnState", ["KilnReady", "KilnFire"])
    BakeStateType, [Unbaked, Baking, Baked] = define_finset_named(problem, "BakeState", ["Unbaked", "Baking", "Baked"])

    kiln_fire_capacity = problem.add_fluent("kiln_fire_capacity", IntType(), kiln=Kiln, default_initial_value=0)

    kiln_state = problem.add_fluent("kiln_state", KilnStateType, kiln=Kiln, default_initial_value=KilnReady)

    kiln_fire_action = DurativeAction("kiln_fire_action", kiln=Kiln)
    kiln_fire_action.set_fixed_duration(20)
    this_state = kiln_state(kiln_fire_action.parameter("kiln"))

    kiln_fire_action.add_condition(StartTiming(), Equals(this_state, KilnReady))
    kiln_fire_action.add_effect(StartTiming(), this_state, KilnFire)
    kiln_fire_action.add_effect(EndTiming(), this_state, KilnReady)

    this_capacity = kiln_fire_capacity(kiln_fire_action.parameter("kiln"))
    kiln_fire_action.add_increase_effect(StartTiming(), this_capacity, 2)
    kiln_fire_action.add_decrease_effect(EndTiming(), this_capacity, 2)

    problem.add_action(kiln_fire_action)

    bake_state = problem.add_fluent("bake_state", BakeStateType, piece=Piece, default_initial_value=Unbaked)

    bake_action = DurativeAction("bake_action", piece=Piece, kiln=Kiln)
    this_state = bake_state(bake_action.parameter("piece"))
    bake_action.set_fixed_duration(5)
    bake_action.add_condition(StartTiming(), Equals(this_state, Unbaked))
    bake_action.add_effect(StartTiming(), this_state, Baking)
    bake_action.add_effect(EndTiming(), this_state, Baked)

    this_capacity = kiln_fire_capacity(bake_action.parameter("kiln"))
    bake_action.add_decrease_effect(StartTiming(), this_capacity, 1)
    bake_action.add_increase_effect(EndTiming(), this_capacity, 1)

    problem.add_action(bake_action)

    for piece in pieces:
        problem.add_goal(Equals(bake_state(piece), Baked))

    return problem


def mk_tms_timeline(n_kilns: int, n_pieces: int):
    problem = Problem("tms")

    piece_param_types = [(5, 2), (8, 3), (11, 1)]

    #
    # USER TYPES
    #

    Kiln, kilns = define_finset_numbered(problem, "Kiln", n_kilns)
    Piece, pieces = define_finset_numbered(problem, "Piece", n_pieces)
    Assembly, assemblies = define_finset_numbered(problem, "Assembly", n_pieces // 2)
    KilnStateType, [KilnReady, KilnFire] = define_finset_named(problem, "KilnState", ["KilnReady", "KilnFire"])

    BakeStateType, [Unbaked, Baking, Baked, Treating, Treated] = define_finset_named(
        problem, "BakeState", ["Unbaked", "Baking", "Baked", "Treating", "Treated"]
    )

    AssemblyStateType, [
        Unassembled,
        Assembling,
        Assembled,
        AssemblyBaking,
        AssemblyBaked,
    ] = define_finset_named(
        problem,
        "AssemblyState",
        ["Unassembled", "Assembling", "Assembled", "AssemblyBaking", "AssemblyBaked"],
    )

    #
    # KILN
    #

    kiln_fire_capacity = problem.add_fluent("kiln_fire_capacity", IntType(), kiln=Kiln, default_initial_value=0)

    kiln_state = problem.add_fluent("kiln_state", KilnStateType, kiln=Kiln, default_initial_value=KilnReady)

    kiln_fire_action = DurativeAction("kiln_fire_action", kiln=Kiln)
    kiln_fire_action.set_fixed_duration(20)
    this_state = kiln_state(kiln_fire_action.parameter("kiln"))

    kiln_fire_action.add_condition(StartTiming(), Equals(this_state, KilnReady))
    kiln_fire_action.add_effect(StartTiming(), this_state, KilnFire)
    kiln_fire_action.add_effect(EndTiming(), this_state, KilnReady)

    this_capacity = kiln_fire_capacity(kiln_fire_action.parameter("kiln"))
    kiln_fire_action.add_increase_effect(StartTiming(), this_capacity, 2)
    kiln_fire_action.add_decrease_effect(EndTiming(), this_capacity, 2)

    problem.add_action(kiln_fire_action)

    #
    # PIECES
    #

    bake_state = problem.add_fluent("bake_state", BakeStateType, piece=Piece, default_initial_value=Unbaked)

    for piece_idx, piece in enumerate(pieces):
        bake_time, treat_time = piece_param_types[piece_idx % len(piece_param_types)]

        bake_action = DurativeAction(f"bake_action({piece.name})", kiln=Kiln)
        this_state = bake_state(piece)
        bake_action.set_fixed_duration(bake_time)
        bake_action.add_condition(StartTiming(), Equals(this_state, Unbaked))
        bake_action.add_effect(StartTiming(), this_state, Baking)
        bake_action.add_effect(EndTiming(), this_state, Baked)

        this_capacity = kiln_fire_capacity(bake_action.parameter("kiln"))
        bake_action.add_decrease_effect(StartTiming(), this_capacity, 1)
        bake_action.add_increase_effect(EndTiming(), this_capacity, 1)
        problem.add_action(bake_action)

        treat_action = DurativeAction(f"treat_action({piece.name})")
        this_state = bake_state(piece)
        treat_action.set_fixed_duration(treat_time)
        treat_action.add_condition(StartTiming(), Equals(this_state, Baked))
        treat_action.add_effect(StartTiming(), this_state, Treating)
        treat_action.add_effect(EndTiming(), this_state, Treated)
        problem.add_action(treat_action)

    #
    # ASSEMBLY
    #

    assembly_state = problem.add_fluent(
        "assembly_state",
        AssemblyStateType,
        assembly=Assembly,
        default_initial_value=Unassembled,
    )

    for assembly_idx, assembly in enumerate(assemblies):
        assemble_action = DurativeAction(f"assemble_action({assembly.name})")
        piece_a = pieces[2 * assembly_idx]
        piece_b = pieces[2 * assembly_idx + 1]
        this_state = assembly_state(assembly)
        assemble_action.set_fixed_duration(1)
        assemble_action.add_condition(StartTiming(), Equals(this_state, Unassembled))
        assemble_action.add_condition(
            ClosedTimeInterval(StartTiming(), EndTiming()), Equals(bake_state(piece_a), Treated)
        )
        assemble_action.add_condition(
            ClosedTimeInterval(StartTiming(), EndTiming()), Equals(bake_state(piece_b), Treated)
        )
        assemble_action.add_effect(StartTiming(), this_state, Assembling)
        assemble_action.add_effect(EndTiming(), this_state, Assembled)
        problem.add_action(assemble_action)

        assembly_bake_action = DurativeAction(f"assembly_bake_action({assembly.name})", kiln=Kiln)
        assembly_bake_action.set_fixed_duration(3)
        assembly_bake_action.add_condition(StartTiming(), Equals(this_state, Assembled))
        assembly_bake_action.add_effect(StartTiming(), this_state, AssemblyBaking)
        assembly_bake_action.add_effect(EndTiming(), this_state, AssemblyBaked)

        this_capacity = kiln_fire_capacity(assembly_bake_action.parameter("kiln"))
        assembly_bake_action.add_decrease_effect(StartTiming(), this_capacity, 1)
        assembly_bake_action.add_increase_effect(EndTiming(), this_capacity, 1)
        problem.add_action(assembly_bake_action)

        problem.add_goal(Equals(this_state, AssemblyBaked))

    return problem


def tms_example_problems() -> List[Problem]:
    problems = []
    for n_kilns, n_pieces in [(1, 2), (2, 4), (2, 6), (4, 6), (1, 10), (5, 10), (2, 16), (3, 16)]:
        problem = mk_tms_timeline(n_kilns, n_pieces)
        problems.append(problem)
    return problem
