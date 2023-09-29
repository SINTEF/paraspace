from typing import Optional, Set, Tuple
import unified_planning as up
from unified_planning.shortcuts import *
from unified_planning.engines.results import PlanGenerationResultStatus
import pyparaspace as paraspace
import itertools
from dataclasses import dataclass

from .conversion_error import ParaspaceTimelinesPlannerConversionError


@dataclass
class ConvValue:
    name: str
    consumes_resources: List[Tuple[str, int]]
    fixed_duration: int | None
    action: Action | None


@dataclass
class ConvTL:
    initial_value: str | None
    goal: str | None
    valid_transitions: Set[Tuple[str, str]]
    values: List[ConvValue]


@dataclass
class ConvResource:
    initial_value: int | None
    provided_by: List[Tuple[str, str, int]]


@dataclass
class Stage1:
    timelines: Dict[str, ConvTL]
    resources: Dict[str, ConvResource]


def do_ground_fluent(fluent: "up.model.fluent", domains) -> List[Fluent]:
    if len(fluent.signature) == 0:
        return [fluent]

    grounded_fluents = []
    all_values = []
    for par in fluent.signature:
        if str(par.type) in domains.keys():
            all_values += [domains[str(par.type)]]
        else:
            raise ParaspaceTimelinesPlannerConversionError(
                "Not supported parameter for grounding fluent"
            )
    all_combinations = list(itertools.product(*all_values))
    for combination in all_combinations:
        fluent_name = f"{fluent.name}(" + ", ".join(combination) + ")"
        grounded_fluents.append(
            Fluent(
                name=fluent_name,
                typename=fluent.type,
                environment=fluent.environment,
            )
        )
    return grounded_fluents


def convert_stage1(problem: Problem) -> Stage1:
    findomains: Dict[str, List[str]] = {
        ut.name: [obj.name for obj in problem.objects(ut)] for ut in problem.user_types
    }
    findomains["bool"] = ["true", "false"]

    def fluent_str(fluent):
        return str(fluent).split(" ")[-1]

    timelines: Dict[str, ConvTL] = {}
    resources: Dict[str, ConvResource] = {}
    for lifted_fluent in problem.fluents:
        for ground_fluent in do_ground_fluent(lifted_fluent, findomains):
            if str(ground_fluent.type) == "integer":
                resources[fluent_str(ground_fluent)] = ConvResource(None, [])
            else:
                timelines[fluent_str(ground_fluent)] = ConvTL(
                    initial_value=None,
                    goal=None,
                    valid_transitions=set(),
                    values=[
                        ConvValue(
                            name=value,
                            consumes_resources=[],
                            fixed_duration=None,
                            action=None,
                        )
                        for value in findomains[str(lifted_fluent.type)]
                    ],
                )

    def set_initial_value(lhs: Fluent, rhs: FNode):
        var_name = fluent_str(lhs)

        if var_name in resources:
            if not rhs.is_int_constant():
                raise ParaspaceTimelinesPlannerConversionError()
            if rhs.constant_value() != 0:
                raise ParaspaceTimelinesPlannerConversionError()
            resources[var_name].initial_value = 0

        elif var_name in timelines:
            if not rhs.is_object_exp():
                raise ParaspaceTimelinesPlannerConversionError()
            timelines[var_name].initial_value = str(rhs.object())

    for lifted_fluent, rhs in problem.fluents_defaults.items():
        for ground_fluent in do_ground_fluent(lifted_fluent, findomains):
            set_initial_value(ground_fluent, rhs)

    if len(problem.initial_defaults) > 0:
        raise ParaspaceTimelinesPlannerConversionError()

    for lhs, rhs in problem.initial_values.items():
        set_initial_value(lhs.fluent(), rhs)

    for goal in problem.goals:
        if not goal.is_equals():
            raise ParaspaceTimelinesPlannerConversionError()

        lhs = fluent_str(goal.args[0])
        rhs = str(goal.args[1])

        if not lhs in timelines:
            raise ParaspaceTimelinesPlannerConversionError()

        timelines[lhs].goal = rhs

    def is_start(bound: Timing):
        return bound.is_from_start() and bound.delay == 0

    def is_end(bound: Timing):
        return bound.is_from_end() and bound.delay == 0

    for action in problem.actions:
        start_provide: Optional[Tuple[str, int]] = None
        end_provide: Optional[Tuple[str, int]] = None
        start_consume: Optional[Tuple[str, int]] = None
        end_consume: Optional[Tuple[str, int]] = None

        timeline: Optional[str] = None
        transition_from: Optional[str] = None
        transition_to_temporary: Optional[str] = None
        transition_to_final: Optional[str] = None

        if not isinstance(action, type(up.model.action.DurativeAction(""))):
            raise ParaspaceTimelinesPlannerConversionError()
        action: unified_planning.model.DurativeAction = action

        if len(action.simulated_effects) > 0:
            raise ParaspaceTimelinesPlannerConversionError()

        for timing, conds in action.conditions.items():
            if not (is_start(timing.lower) and is_start(timing.upper)):
                raise ParaSpaceTimelinesPlannerConversionError()

            for cond in conds:
                cond: FNode = cond
                if not cond.is_equals():
                    raise ParaSpaceTimelinesPlannerConversionError()
                if timeline is None:
                    timeline = fluent_str(cond.args[0])
                if str(cond.args[0]) != timeline:
                    raise ParaSpaceTimelinesPlannerConversionError()
                if transition_from is None:
                    transition_from = fluent_str(cond.args[1])
                if str(cond.args[1]) != transition_from:
                    raise ParaSpaceTimelinesPlannerConversionError()

        for timing, effs in action.effects.items():
            for eff in effs:
                eff: Effect = eff
                # fluent: Fluent = eff.fluent.fluent()

                if not (eff.condition is None or eff.condition.is_true()):
                    raise ParaSpaceTimelinesPlannerConversionError()

                if eff.kind == EffectKind.INCREASE:
                    if (
                        not str(eff.fluent.type) == "integer"
                        or not eff.value.is_int_constant()
                    ):
                        raise ParaSpaceTimelinesPlannerConversionError()

                    resource_name = fluent_str(eff.fluent)
                    if resource_name not in resources:
                        raise ParaSpaceTimelinesPlannerConversionError()

                    if is_start(timing):
                        start_provide = (resource_name, eff.value.constant_value())
                    elif is_end(timing):
                        end_consume = (resource_name, eff.value.constant_value())
                    else:
                        raise ParaSpaceTimelinesPlannerConversionError()

                elif eff.kind == EffectKind.DECREASE:
                    if (
                        not str(eff.fluent.type) == "integer"
                        or not eff.value.is_int_constant()
                    ):
                        raise ParaSpaceTimelinesPlannerConversionError()

                    resource_name = fluent_str(eff.fluent)
                    if resource_name not in resources:
                        raise ParaSpaceTimelinesPlannerConversionError()

                    if is_start(timing):
                        start_consume = (resource_name, eff.value.constant_value())
                    elif is_end(timing):
                        end_provide = (resource_name, eff.value.constant_value())
                    else:
                        raise ParaSpaceTimelinesPlannerConversionError()

                elif eff.kind == EffectKind.ASSIGN:
                    if not str(eff.fluent.type) in findomains:
                        raise ParaSpaceTimelinesPlannerConversionError()

                    if is_start(timing):
                        if timeline is None:
                            timeline = fluent_str(eff.fluent)
                        if timeline != fluent_str(eff.fluent):
                            raise ParaSpaceTimelinesPlannerConversionError()

                        if transition_to_temporary is None:
                            transition_to_temporary = str(eff.value)
                        if transition_to_temporary != str(eff.value):
                            raise ParaSpaceTimelinesPlannerConversionError()

                    elif is_end(timing):
                        if timeline is None:
                            timeline = fluent_str(eff.fluent)
                        if timeline != fluent_str(eff.fluent):
                            raise ParaSpaceTimelinesPlannerConversionError()

                        if transition_to_final is None:
                            transition_to_final = str(eff.value)
                        if transition_to_final != str(eff.value):
                            raise ParaSpaceTimelinesPlannerConversionError()

        dur: DurationInterval = action.duration
        if not dur.lower.is_int_constant() or not dur.upper.is_int_constant():
            raise ParaSpaceTimelinesPlannerConversionError()

        d1 = dur.lower.constant_value()
        d2 = dur.lower.constant_value()
        if d1 != d2:
            raise ParaSpaceTimelinesPlannerConversionError()

        if (
            timelines is None
            or transition_from is None
            or transition_to_temporary is None
            or transition_to_final is None
        ):
            raise ParaSpaceTimelinesPlannerConversionError()

        timelines[timeline].valid_transitions.add(
            (transition_from, transition_to_temporary)
        )
        timelines[timeline].valid_transitions.add(
            (transition_to_temporary, transition_to_final)
        )

        rhs = next(
            (
                v
                for v in timelines[timeline].values
                if v.name == transition_to_temporary
            ),
            None,
        )

        if rhs is None:
            raise ParaSpaceTimelinesPlannerConversionError()

        rhs.fixed_duration = d1
        rhs.action = action

        # resource use consistency check
        if start_provide != end_provide or start_consume != end_consume:
            raise ParaSpaceTimelinesPlannerConversionError()

        if start_provide is not None:
            res, amount = start_provide
            resources[res].provided_by.append(
                (timeline, transition_to_temporary, amount)
            )

        if start_consume is not None:
            res, amount = start_consume
            rhs.consumes_resources.append((res, amount))

    return Stage1(timelines, resources)


def convert_stage2(problem: Stage1) -> paraspace.Problem:
    assert all(len(res.provided_by) == 1 for res in problem.resources.values())

    timelines = []
    for tl_name, timeline in problem.timelines.items():
        token_types = []
        static_tokens = []

        if timeline.initial_value is not None:
            static_tokens.append(
                paraspace.StaticToken(
                    value=timeline.initial_value,
                    const_time=paraspace.fact(0, None),
                    capacity=0,
                    conditions=[],
                )
            )

        if timeline.goal is not None:
            static_tokens.append(
                paraspace.StaticToken(
                    value=timeline.goal,
                    const_time=paraspace.goal(),
                    capacity=0,
                    conditions=[],
                )
            )

        for value in timeline.values:
            dur = value.fixed_duration

            conditions = []
            resource_alts = []
            for resource_name, amount in value.consumes_resources:
                other_tl, other_value, _provided_amount = problem.resources[
                    resource_name
                ].provided_by[0]
                resource_alts.append(
                    paraspace.TemporalCond(
                        other_tl, other_value, paraspace.TemporalRelation.Cover, amount
                    )
                )

            if len(resource_alts) > 0:
                conditions.append(paraspace.OrCond(resource_alts))

            capacity = 0
            for resource_name, resource in problem.resources.items():
                for provide_tl, provide_value, amount in resource.provided_by:
                    if provide_tl == tl_name and provide_value == value.name:
                        capacity = amount

            transition_from = []
            transition_to = []
            for a, b in timeline.valid_transitions:
                if a == value.name:
                    transition_to.append(b)
                if b == value.name:
                    transition_from.append(a)

            if len(transition_from) > 0:
                conditions.append(
                    paraspace.OrCond(
                        [
                            paraspace.TemporalCond(
                                tl_name,
                                other_value,
                                paraspace.TemporalRelation.MetBy,
                                0,
                            )
                            for other_value in transition_from
                        ]
                    )
                )

            if len(transition_to) > 0 and dur is not None:
                conditions.append(
                    paraspace.OrCond(
                        [
                            paraspace.TemporalCond(
                                tl_name,
                                other_value,
                                paraspace.TemporalRelation.Meets,
                                0,
                            )
                            for other_value in transition_to
                        ]
                    )
                )

            token_types.append(
                paraspace.TokenType(
                    value=value.name,
                    conditions=conditions,
                    duration_limits=(1, None) if dur is None else (dur, dur),
                    capacity=capacity,
                )
            )

        timelines.append(
            paraspace.Timeline(
                name=tl_name, token_types=token_types, static_tokens=static_tokens
            )
        )
    return paraspace.Problem(timelines=timelines)


class ParaspaceTimelinesProblemConversion:
    def __init__(self, problem, compiler_res):
        self.problem = problem
        self.compiler_res = compiler_res
        self.return_time_triggered_plan = False
