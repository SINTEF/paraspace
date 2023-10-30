from typing import Optional, Set, Tuple
import unified_planning as up
from unified_planning.shortcuts import *
from unified_planning.engines.results import PlanGenerationResultStatus
import pyparaspace as paraspace
import itertools
from dataclasses import dataclass

from .conversion_error import ParaspaceTimelinesPlannerConversionError


@dataclass
class ConvActionMap:
    resources: List[str]
    during: List[Tuple[str, str]]
    action: Action


@dataclass
class ConvValue:
    name: str
    consumes_resources: List[Tuple[str, int]]
    during_other_token: List[Tuple[str, str]]
    fixed_duration: int | None
    actions: List[ConvActionMap]


@dataclass
class ConvTL:
    initial_values :List[Tuple[int,str]]
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
            raise ParaspaceTimelinesPlannerConversionError("Not supported parameter for grounding fluent")
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
    findomains: Dict[str, List[str]] = {ut.name: [obj.name for obj in problem.objects(ut)] for ut in problem.user_types}
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
                    initial_values=[],
                    goal=None,
                    valid_transitions=set(),
                    values=[
                        ConvValue(
                            name=value,
                            consumes_resources=[],
                            during_other_token=[],
                            fixed_duration=None,
                            actions=[],
                        )
                        for value in findomains[str(lifted_fluent.type)]
                    ],
                )

    def set_initial_value(lhs: Fluent, rhs: FNode):
        var_name = fluent_str(lhs)

        if var_name in resources:
            if not rhs.is_int_constant():
                raise ParaspaceTimelinesPlannerConversionError("Initial value for resource must be integer.")
            if rhs.constant_value() != 0:
                raise ParaspaceTimelinesPlannerConversionError("Initial value for resource must be zero.")
            resources[var_name].initial_value = 0

        elif var_name in timelines:
            tl = timelines[var_name]
            if not rhs.is_object_exp():
                raise ParaspaceTimelinesPlannerConversionError("Initial value of timeline fluent must be an object.")
            tl.initial_values = [(t,v) for t,v in tl.initial_values if t != 0]
            tl.initial_values.append((0,str(rhs.object())))

    for lifted_fluent, value in problem.fluents_defaults.items():
        for ground_fluent in do_ground_fluent(lifted_fluent, findomains):
            set_initial_value(ground_fluent, value)

    if len(problem.initial_defaults) > 0:
        raise ParaspaceTimelinesPlannerConversionError("Initial default values are not supported.")

    if len(problem.timed_goals) > 0:
        raise ParaspaceTimelinesPlannerConversionError("Timed goals values are not supported.")


    for lhs, value in problem.initial_values.items():
        set_initial_value(lhs.fluent(), value)


    for timing,effs in problem.timed_effects.items():
        if not (timing.is_global() and timing.is_from_start()):
            raise ParaspaceTimelinesPlannerConversionError("Timed effects must be relative to global start time.")
        
        for eff in effs:
            if not eff.is_assignment():
                raise ParaspaceTimelinesPlannerConversionError("Timed effects must be simple assignments.")

            fluent = fluent_str(eff.fluent)
            value = str(eff.value)

            if not fluent in timelines:
                raise ParaspaceTimelinesPlannerConversionError("Timed effects must apply only to timeline fluents.")
            
            if not any((t == timing.delay for t,_ in timelines[fluent].initial_values)):
                timelines[fluent].initial_values.append((timing.delay, value ))

    for goal in problem.goals:
        if not goal.is_equals():
            raise ParaspaceTimelinesPlannerConversionError(
                "Only equality goals between timeline fluent and an object are supported."
            )

        lhs = fluent_str(goal.args[0])
        value = str(goal.args[1])

        if not lhs in timelines:
            raise ParaspaceTimelinesPlannerConversionError("Only equality goals for timeline fluents are supported.")

        timelines[lhs].goal = value

    def is_start(bound: Timing):
        return bound.is_from_start() and bound.delay == 0

    def is_end(bound: Timing):
        return bound.is_from_end() and bound.delay == 0

    for action in problem.actions:
        # print(action)
        start_provide: Optional[Tuple[str, int]] = None
        end_provide: Optional[Tuple[str, int]] = None
        start_consume: Optional[Tuple[str, int]] = None
        end_consume: Optional[Tuple[str, int]] = None

        timeline: Optional[str] = None
        transition_from: Optional[str] = None
        transition_to_temporary: Optional[str] = None
        transition_to_final: Optional[str] = None

        synchronization: List[Tuple[str, str]] = []

        if not isinstance(action, type(up.model.action.DurativeAction(""))):
            raise ParaspaceTimelinesPlannerConversionError("Only durative actions are supported.")
        action: unified_planning.model.DurativeAction = action

        if len(action.simulated_effects) > 0:
            raise ParaspaceTimelinesPlannerConversionError("Simulated effects are not supported.")

        for timing, conds in action.conditions.items():
            if is_start(timing.lower) and is_end(timing.upper):
                # "During" synchronization constraint

                for cond in conds:
                    if not cond.is_equals():
                        raise ParaspaceTimelinesPlannerConversionError("Only equality conditions are supported")

                    other_tl = str(cond.args[0])
                    other_value = str(cond.args[1])

                    if not other_tl in timelines or not other_value in (v.name for v in timelines[other_tl].values):
                        raise ParaspaceTimelinesPlannerConversionError(
                            "Did not recognize timeline value for synchronization constraint."
                        )

                    synchronization.append((other_tl, other_value))

            elif is_start(timing.lower) and is_start(timing.upper):
                for cond in conds:
                    cond: FNode = cond
                    if not cond.is_equals():
                        raise ParaspaceTimelinesPlannerConversionError("Only equality conditions are supported")
                    if timeline is None:
                        timeline = fluent_str(cond.args[0])
                    if str(cond.args[0]) != timeline:
                        raise ParaspaceTimelinesPlannerConversionError("Ambiguous timeline for action condition.")
                    if transition_from is None:
                        transition_from = fluent_str(cond.args[1])
                    if str(cond.args[1]) != transition_from:
                        raise ParaspaceTimelinesPlannerConversionError("Ambiguous timeline value for action condition.")
            else:
                raise ParaspaceTimelinesPlannerConversionError(
                    "Action condition must apply at the start of, or during, durative action."
                )

        for timing, effs in action.effects.items():
            for eff in effs:
                eff: Effect = eff
                # fluent: Fluent = eff.fluent.fluent()

                if not (eff.condition is None or eff.condition.is_true()):
                    raise ParaspaceTimelinesPlannerConversionError("Conditional effects are not supported.")

                if eff.kind == EffectKind.INCREASE:
                    if not str(eff.fluent.type) == "integer" or not eff.value.is_int_constant():
                        raise ParaspaceTimelinesPlannerConversionError(
                            "Increase effects must be applied as constants to integer fluents."
                        )

                    resource_name = fluent_str(eff.fluent)
                    if resource_name not in resources:
                        raise ParaspaceTimelinesPlannerConversionError(
                            "Did not recognize resource for increase effect."
                        )

                    if is_start(timing):
                        start_provide = (resource_name, eff.value.constant_value())
                    elif is_end(timing):
                        end_consume = (resource_name, eff.value.constant_value())
                    else:
                        raise ParaspaceTimelinesPlannerConversionError(
                            "Increase effect must apply at start or end of durative action."
                        )

                elif eff.kind == EffectKind.DECREASE:
                    if not str(eff.fluent.type) == "integer" or not eff.value.is_int_constant():
                        raise ParaspaceTimelinesPlannerConversionError(
                            "Decrease effects must be applied as constants to integer fluents."
                        )

                    resource_name = fluent_str(eff.fluent)
                    if resource_name not in resources:
                        raise ParaspaceTimelinesPlannerConversionError(
                            "Did not recognize resource for decrease effect."
                        )

                    if is_start(timing):
                        start_consume = (resource_name, eff.value.constant_value())
                    elif is_end(timing):
                        end_provide = (resource_name, eff.value.constant_value())
                    else:
                        raise ParaspaceTimelinesPlannerConversionError(
                            "Decrease effect must apply at start or end of durative action."
                        )

                elif eff.kind == EffectKind.ASSIGN:
                    if not str(eff.fluent.type) in findomains:
                        raise ParaspaceTimelinesPlannerConversionError(
                            "Assign effects must apply to recognized timelines."
                        )

                    if is_start(timing):
                        if timeline is None:
                            timeline = fluent_str(eff.fluent)
                        if timeline != fluent_str(eff.fluent):
                            raise ParaspaceTimelinesPlannerConversionError("Ambiguous timeline for action effect.")

                        if transition_to_temporary is None:
                            transition_to_temporary = str(eff.value)
                        if transition_to_temporary != str(eff.value):
                            raise ParaspaceTimelinesPlannerConversionError(
                                "Ambiguous timeline value for action effect."
                            )

                    elif is_end(timing):
                        if timeline is None:
                            timeline = fluent_str(eff.fluent)
                        if timeline != fluent_str(eff.fluent):
                            raise ParaspaceTimelinesPlannerConversionError("Ambiguous timeline for action effect.")

                        if transition_to_final is None:
                            transition_to_final = str(eff.value)
                        if transition_to_final != str(eff.value):
                            raise ParaspaceTimelinesPlannerConversionError(
                                "Ambiguous timeline value for action effect."
                            )

        dur: DurationInterval = action.duration
        if not dur.lower.is_int_constant() or not dur.upper.is_int_constant():
            raise ParaspaceTimelinesPlannerConversionError("Only fixed constant action durations are supported.")

        d1 = dur.lower.constant_value()
        d2 = dur.lower.constant_value()
        if d1 != d2:
            raise ParaspaceTimelinesPlannerConversionError("Only fixed constant action durations are supported.")

        if (
            timelines is None
            or transition_from is None
            or transition_to_temporary is None
            or transition_to_final is None
        ):
            raise ParaspaceTimelinesPlannerConversionError("Did not recognize action as timeline transition.")

        timelines[timeline].valid_transitions.add((transition_from, transition_to_temporary))
        timelines[timeline].valid_transitions.add((transition_to_temporary, transition_to_final))

        value = next(
            (v for v in timelines[timeline].values if v.name == transition_to_temporary),
            None,
        )

        if value is None:
            raise ParaspaceTimelinesPlannerConversionError(f"Unknown timeline value '{transition_to_temporary}'.")

        value.fixed_duration = d1

        value.during_other_token += synchronization

        # resource use consistency check
        if start_provide != end_provide or start_consume != end_consume:
            raise ParaspaceTimelinesPlannerConversionError("Did not recognize consistent resource use/provide effects.")

        if start_provide is not None:
            res, amount = start_provide
            resources[res].provided_by.append((timeline, transition_to_temporary, amount))

        required_resources = []
        if start_consume is not None:
            res, amount = start_consume
            required_resources.append(res)
            value.consumes_resources.append((res, amount))

        value.actions.append(ConvActionMap(resources=required_resources, during=list(synchronization), action=action))

    # # Convert ConvActionMap resources references into during-constraints.
    for tl_name, tl in timelines.items():
        for value in tl.values:
            for a in value.actions:
                for resource in a.resources:
                    resource_spec = resources[resource]
                    assert len(resource_spec.provided_by) == 1
                    for ot, ov, _amount in resource_spec.provided_by:
                        a.during.append((ot, ov))
                a.resources = []

    # print("RESOURCES", resources)
    # print("TIMELINES", timelines)

    for timeline in timelines.values():
        timeline.initial_values.sort(key=lambda x: x[0])

    return Stage1(timelines, resources)


def convert_stage2(problem: Stage1) -> paraspace.Problem:
    assert all(len(res.provided_by) == 1 for res in problem.resources.values())

    timelines = []
    for tl_name, timeline in problem.timelines.items():
        token_types = []
        static_tokens = []

        for idx,(this_time,val) in enumerate(timeline.initial_values):
            next_time = timeline.initial_values[idx+1][0] if idx+1 < len(timeline.initial_values) else None
            static_tokens.append(
                paraspace.StaticToken(
                    value=val,
                    const_time=paraspace.fact(this_time, next_time),
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
                other_tl, other_value, _provided_amount = problem.resources[resource_name].provided_by[0]
                resource_alts.append(
                    paraspace.TemporalCond(other_tl, other_value, paraspace.TemporalRelation.Cover, amount)
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

            if len(transition_from) == 0:
                continue  # no valid transitions into this state

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

            for other_tl, other_value in value.during_other_token:
                conditions.append(
                    paraspace.OrCond(
                        [paraspace.TemporalCond(other_tl, other_value, paraspace.TemporalRelation.Cover, 0)]
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

        timelines.append(paraspace.Timeline(name=tl_name, token_types=token_types, static_tokens=static_tokens))
    return paraspace.Problem(timelines=timelines)


class ParaspaceTimelinesProblemConversion:
    def __init__(self, problem, compiler_res):
        self.problem = problem
        self.compiler_res = compiler_res

    def _decompile(self, conv_problem: Stage1, solution):
        actions = []
        tl_by_name = {tl.name: tl for tl in solution.timelines}

        for tl in solution.timelines:
            if not tl.name in conv_problem.timelines:
                raise ParaspaceTimelinesPlannerConversionError(f"Received unknown timeline from solver '{tl.name}'.")

            tl_spec = conv_problem.timelines[tl.name]
            for token in tl.tokens:
                t = token.start_time
                dur = token.end_time - token.start_time

                value = next((v for v in tl_spec.values if v.name == token.value), None)
                if value is None:
                    raise ParaspaceTimelinesPlannerConversionError(
                        f"Received unknown timeline value from solver '{tl.name}.{token.value}'."
                    )

                conditions = set(
                    (
                        (cond.timeline, tl_by_name[cond.timeline].tokens[cond.token_idx].value)
                        for cond in token.conditions
                    )
                )

                for action_spec in value.actions:
                    if all((otl, ov) in conditions for otl, ov in action_spec.during):
                        instance = up.plans.ActionInstance(action_spec.action)
                        actions.append((t, instance, dur))

        plan = up.plans.time_triggered_plan.TimeTriggeredPlan(actions)
        for res in reversed(self.compiler_res):
            plan = plan.replace_action_instances(res.map_back_action_instance)
        return plan

    def convert_and_solve(self, problem: "up.model.Problem"):
        stage1 = convert_stage1(problem)
        paraspace_problem = convert_stage2(stage1)
        solution = paraspace.solve(paraspace_problem)
        plan = self._decompile(stage1, solution)
        return plan
        
