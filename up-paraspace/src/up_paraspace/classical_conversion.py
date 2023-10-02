from typing import Optional, Callable, IO
from unified_planning.engines.results import PlanGenerationResultStatus
import unified_planning as up
import unified_planning.engines as engines
from unified_planning.shortcuts import *
import pyparaspace as pps
from collections import defaultdict
import itertools
from fractions import Fraction
from .conversion_error import ParaspaceTimelinesPlannerConversionError

def type_name(t):
    if t.is_bool_type():
        return "bool"
    elif t.is_user_type():
        return t.name
    else:
        raise ParaspaceTimelinesPlannerConversionError("unsupported type")


class ParaspaceClassicalProblemConversion:
    def __init__(self, problem, compiler_res):
        self.problem = problem
        self.compiler_res = compiler_res
        self.return_time_triggered_plan = False

    def _ground_fluent(self, fluent: "up.model.fluent", domains):
        grounded_fluents = []
        all_values = []
        for par in fluent.signature:
            if str(par.type) in domains.keys():
                all_values += [domains[str(par.type)]]
            else:
                print("Not supported parameter for grounding fluent")
        all_combinations = list(itertools.product(*all_values))
        for combination in all_combinations:
            fluent_name = fluent.name + "(" + combination[0]
            if len(combination) > 1:
                for c in combination[1:]:
                    fluent_name += ", " + c
            fluent_name += ")"
            grounded_fluents.append(
                Fluent(
                    name=fluent_name,
                    typename=fluent.type,
                    environment=fluent.environment,
                )
            )
        return grounded_fluents

    def _convert_solution(self, solution):
        if self.return_time_triggered_plan:
            actions = []
            for timeline in solution.timelines:
                if timeline.name == "actions":
                    for token in timeline.tokens:
                        if not token.value == "wAiT":  # Remove syntatic wait action
                            actions.append(
                                (
                                    Fraction(token.start_time),
                                    up.plans.plan.ActionInstance(
                                        self.action_map[token.value]
                                    ),
                                    Fraction(token.end_time - token.start_time),
                                )
                            )
            plan = up.plans.time_triggered_plan.TimeTriggeredPlan(actions)
        else:
            actions = []
            for timeline in solution.timelines:
                if timeline.name == "actions":
                    for token in timeline.tokens:
                        actions.append(
                            up.plans.plan.ActionInstance(self.action_map[token.value])
                        )
            plan = up.plans.sequential_plan.SequentialPlan(actions)
        for res in reversed(self.compiler_res):
            plan = plan.replace_action_instances(res.map_back_action_instance)
        return plan

    def _decode_duration(self, action, problem):
        statics = problem.get_static_fluents()
        dur_lower = 0
        if action.duration.lower.is_int_constant():
            dur_lower = action.duration.lower.int_constant_value()
        elif action.duration.lower.is_fluent_exp():
            init_value = problem.initial_value(action.duration.lower)
            if (
                action.duration.lower.fluent() in statics
                and init_value.is_int_constant()
            ):
                dur_lower = init_value.int_constant_value()
            else:
                raise ParaspaceTimelinesPlannerConversionError("Not supported duration fluent", action.duration)
        else:
            raise ParaspaceTimelinesPlannerConversionError("Not supported duration type", action.duration)
        dur_upper = 0
        if action.duration.upper.is_int_constant():
            dur_upper = action.duration.upper.int_constant_value()
        elif action.duration.upper.is_fluent_exp():
            init_value = problem.initial_value(action.duration.upper)
            if (
                action.duration.upper.fluent() in statics
                and init_value.is_int_constant()
            ):
                dur_upper = init_value.int_constant_value()
            else:
                raise ParaspaceTimelinesPlannerConversionError("Not supported duration fluent", action.duration)
        else:
            raise ParaspaceTimelinesPlannerConversionError("Not supported duration type", action.duration)
        if dur_lower < 1 and dur_upper < 1:
            raise ParaspaceTimelinesPlannerConversionError("Do only support actions with duration longer than one")
        return (dur_lower, dur_upper)

    def _decode_fnode(self, fnode, domains):
        if fnode.type.is_bool_type():
            if fnode.is_equals():
                args = list(fnode.args)
                if len(args) != 2:
                    raise ParaspaceTimelinesPlannerConversionError("Not supported Equal expression without 2 arguments")
                timeline = str(args[0])
                value = str(args[1])
                fluent_type = str(args[0].fluent().type)
                return [(timeline, value, fluent_type)]
            elif fnode.is_fluent_exp():
                if fnode.fluent().type.is_bool_type():
                    timeline = str(fnode)
                    fluent_type = str(fnode.fluent().type)
                    return [(timeline, "true", fluent_type)]
                else:
                    raise ParaspaceTimelinesPlannerConversionError("Not supported type" + str(fnode.node_type) + str(
                        fnode.fluent().name
                    ))
            elif fnode.is_not():
                if len(fnode.args) > 1:
                    raise ParaspaceTimelinesPlannerConversionError("Only supports one exp inside NOT exp")
                for arg in fnode.args:
                    [(timeline, value, fluent_type)] = self._decode_fnode(arg, domains)
                    nodes = []
                    for v in domains[fluent_type]:
                        if v != value:
                            nodes.append((timeline, v, fluent_type))
                    return nodes
            else:
                raise ParaspaceTimelinesPlannerConversionError(
                    "Not supported precondition operator kind" + str(fnode.node_type)
                )
        else:
            raise ParaspaceTimelinesPlannerConversionError("Not supported precondtion type" + str(fnode.type))
        return None

    def convert_and_solve(self, problem: "up.model.Problem"):
        #
        # CONVERT
        # Convert each element of the planning problem to pyparaspace objects.
        #

        domains = {
            ut.name: [obj.name for obj in problem.objects(ut)]
            for ut in problem.user_types
        }
        domains["bool"] = ["true", "false"]

        print(problem)

        # print("DOMAINS\n",domains)
        timelines = []
        statics = problem.get_static_fluents()
        action_values = []
        frame_cond = []
        self.action_map = {}
        has_wAiT_action = False
        ## Make an action timeline
        for action in problem.actions:
            self.action_map[action.name] = action
            conditions = []
            preconditions = []
            if isinstance(action, type(up.model.action.DurativeAction(""))):
                self.return_time_triggered_plan = True
                duration = self._decode_duration(action, problem)
                if action.simulated_effects != {}:
                    raise ParaspaceTimelinesPlannerConversionError("Do not support simulated effects")
                for timing, conds in action.conditions.items():
                    if str(timing.lower) == str(timing.upper) == "start":
                        for pre in conds:
                            decoded_pre = self._decode_fnode(pre, domains)
                            or_conds = []
                            for d_ii in decoded_pre:
                                timeline = d_ii[0]
                                value = d_ii[1]
                                if not (timeline, value) in preconditions:
                                    preconditions.append((timeline, value))
                                    or_conds.append(
                                        pps.TemporalCond(
                                            timeline,
                                            value,
                                            pps.TemporalRelation.StartPrecond,
                                            0,
                                        )
                                    )
                            if len(or_conds) > 0:
                                conditions.append(pps.OrCond(or_conds))
                    else:
                        raise ParaspaceTimelinesPlannerConversionError("Not supported timing kind", timing)
                effs_from_start = []
                effs_from_end = []
                for timing, effs in action.effects.items():
                    if timing.is_from_start():
                        effs_from_start += effs
                    elif timing.is_from_end():
                        has_wAiT_action = True
                        effs_from_end += effs
                    else:
                        raise ParaspaceTimelinesPlannerConversionError("Not supported Timing kind", timing)
                for eff in effs_from_start:
                    if eff.is_assignment():
                        timeline = str(eff.fluent)
                        value = str(eff.value)
                        conditions.append(
                            pps.TemporalCond(
                                timeline, value, pps.TemporalRelation.StartEffect, 0
                            )
                        )
                        from_value = []
                        for pre_ii in preconditions:
                            if pre_ii[0] == timeline:
                                from_value.append(pre_ii[1])
                        frame_cond.append(
                            {
                                "action": action.name,
                                "timeline": timeline,
                                "value": value,
                                "from_value": from_value,
                                "tempRel": pps.TemporalRelation.Starts,
                            }
                        )
                    else:
                        raise ParaspaceTimelinesPlannerConversionError("Effect kind not supported" + str(eff.kind))
                for eff in effs_from_end:
                    if eff.is_assignment():
                        timeline = str(eff.fluent)
                        value = str(eff.value)
                        conditions.append(
                            pps.TemporalCond(
                                timeline, value, pps.TemporalRelation.Meets, 0
                            )
                        )
                        from_value = []
                        for eff_ii in effs_from_start:
                            if str(eff_ii.fluent) == timeline:
                                from_value.append(str(eff_ii.value))
                        if len(from_value) == 0:
                            for pre_ii in preconditions:
                                if pre_ii[0] == timeline:
                                    from_value.append(pre_ii[1])
                        frame_cond.append(
                            {
                                "action": action.name,
                                "timeline": timeline,
                                "value": value,
                                "from_value": from_value,
                                "tempRel": pps.TemporalRelation.MetBy,
                            }
                        )
                action_values.append(
                    pps.TokenType(
                        value=action.name,
                        conditions=conditions,
                        duration_limits=duration,
                        capacity=0,
                    )
                )
            else:
                for pre in action.preconditions:
                    decoded_pre = self._decode_fnode(pre, domains)
                    or_conds = []
                    for d_ii in decoded_pre:
                        timeline = d_ii[0]
                        value = d_ii[1]
                        if not (timeline, value) in preconditions:
                            preconditions.append((timeline, value))
                            or_conds.append(
                                pps.TemporalCond(
                                    timeline,
                                    value,
                                    pps.TemporalRelation.StartPrecond,
                                    0,
                                )
                            )
                    if len(or_conds) > 0:
                        conditions.append(pps.OrCond(or_conds))
                for eff in action.effects:
                    if eff.is_assignment():
                        timeline = str(eff.fluent)
                        value = str(eff.value)
                        conditions.append(
                            pps.TemporalCond(
                                timeline, value, pps.TemporalRelation.StartEffect, 0
                            )
                        )
                        from_value = []
                        for pre_ii in preconditions:
                            if pre_ii[0] == timeline:
                                from_value.append(pre_ii[1])
                        frame_cond.append(
                            {
                                "action": action.name,
                                "timeline": timeline,
                                "value": value,
                                "from_value": from_value,
                                "tempRel": pps.TemporalRelation.Starts,
                            }
                        )
                    else:
                        raise ParaspaceTimelinesPlannerConversionError("Effect kind not supported" + str(eff.kind))
                action_values.append(
                    pps.TokenType(
                        value=action.name,
                        conditions=conditions,
                        duration_limits=(1, None),
                        capacity=0,
                    )
                )
        if has_wAiT_action:
            action_values.append(
                pps.TokenType(
                    value="wAiT", conditions=[], duration_limits=(1, None), capacity=0
                )
            )
        # Make an action timeline
        timelines.append(
            pps.Timeline(name="actions", token_types=action_values, static_tokens=[])
        )
        ## Add Goals and Facts
        inits = problem.initial_values
        static_tokens = defaultdict(list)
        for key in inits:
            static_tokens[str(key)].append(
                pps.StaticToken(
                    value=str(inits[key]),
                    const_time=pps.fact(0, None),
                    capacity=0,
                    conditions=[],
                )
            )
        for goal in problem.goals:
            if goal.is_equals():
                timeline = ""
                value = ""
                for arg in goal.args:
                    if arg.is_fluent_exp():
                        timeline = str(arg)
                    elif arg.is_object_exp():
                        value = arg.object().name
                    else:
                        raise ParaspaceTimelinesPlannerConversionError("Not yet supported arguments type" + str(arg.node_type))
                static_tokens[timeline].append(
                    pps.StaticToken(
                        value=value, const_time=pps.goal(), capacity=0, conditions=[]
                    )
                )
            elif goal.is_fluent_exp():
                static_tokens[str(goal)].append(
                    pps.StaticToken(
                        value="true", const_time=pps.goal(), capacity=0, conditions=[]
                    )
                )
            elif goal.is_not() and goal.args[0].is_fluent_exp():
                static_tokens[str(goal.args[0])].append(
                    pps.StaticToken(
                        value="false", const_time=pps.goal(), capacity=0, conditions=[]
                    )
                )
            else:
                raise ParaspaceTimelinesPlannerConversionError(f"Not supported goal type" + str(goal.node_type))

        # Sort out the frame_axioms
        frame_cond_dict = {}
        for ax in frame_cond:
            if ax["timeline"] in frame_cond_dict.keys():
                if ax["value"] in frame_cond_dict[ax["timeline"]].keys():
                    frame_cond_dict[ax["timeline"]][ax["value"]]["actions"].append(
                        (ax["action"], ax["tempRel"])
                    )
                    if (
                        not ax["from_value"]
                        in frame_cond_dict[ax["timeline"]][ax["value"]]["dtg"]
                    ):
                        frame_cond_dict[ax["timeline"]][ax["value"]]["dtg"] += ax[
                            "from_value"
                        ]
                else:
                    frame_cond_dict[ax["timeline"]][ax["value"]] = {
                        "actions": [(ax["action"], ax["tempRel"])],
                        "dtg": ax["from_value"],
                    }
            else:
                frame_cond_dict[ax["timeline"]] = {
                    ax["value"]: {
                        "actions": [(ax["action"], ax["tempRel"])],
                        "dtg": ax["from_value"],
                    }
                }

        print(frame_cond_dict)
        # Add all fluents as timelines
        fluents = problem.fluents
        for fluent in problem.fluents:
            if not fluent in statics:
                values = []
                if str(fluent.type) in domains.keys():
                    values = domains[str(fluent.type)]
                else:
                    raise ParaspaceTimelinesPlannerConversionError("TODO: Continous-time fluent types is not supported -> resources")

                # Ground fluents
                if len(fluent.signature) != 0:
                    fs = self._ground_fluent(fluent, domains)
                    print("GROUND FLUENT", fs)
                    fluent = fs[0]
                    fs.pop(0)
                    fluents += fs
                token_types = []
                for value in values:
                    if fluent.name in frame_cond_dict.keys():
                        actions_with_this_effect = []
                        possible_previous_values = []
                        if value in frame_cond_dict[fluent.name].keys():
                            for a, temp_rel in frame_cond_dict[fluent.name][value][
                                "actions"
                            ]:
                                actions_with_this_effect.append(
                                    pps.TemporalCond("actions", a, temp_rel, 0)
                                )
                            for v_ii in frame_cond_dict[fluent.name][value]["dtg"]:
                                possible_previous_values.append(
                                    pps.TemporalCond(
                                        fluent.name, v_ii, pps.TemporalRelation.MetBy, 0
                                    )
                                )

                            if len(possible_previous_values) == 0:
                                token_types.append(
                                    pps.TokenType(
                                        value=value,
                                        conditions=[
                                            pps.OrCond(actions_with_this_effect)
                                        ],
                                        duration_limits=(1, None),
                                        capacity=0,
                                    )
                                )
                            else:
                                token_types.append(
                                    pps.TokenType(
                                        value=value,
                                        conditions=[
                                            pps.OrCond(actions_with_this_effect),
                                            pps.OrCond(possible_previous_values),
                                        ],
                                        duration_limits=(1, None),
                                        capacity=0,
                                    )
                                )

                timelines += [
                    pps.Timeline(
                        name=fluent.name,
                        token_types=token_types,
                        static_tokens=static_tokens[fluent.name],
                    )
                ]

        problem_tl = pps.Problem(timelines=timelines)

        for tl in problem_tl.timelines:
            print("-------------------")
            print("TIMELINE:", tl.name)
            for vii in tl.token_types:
                print("token type:", vii.value)
                print("  conditions:")
                for cii in vii.conditions:
                    if isinstance(cii, pps.OrCond):
                        print("      ", "---")
                        for dii in cii.disjuncts:
                            print(
                                "      ",
                                "OR ",
                                str(dii.temporal_relation)[17:],
                                " : ",
                                dii.timeline,
                                " == ",
                                dii.value,
                            )
                    else:
                        print(
                            "      ",
                            str(cii.temporal_relation)[17:],
                            " : ",
                            cii.timeline,
                            " == ",
                            cii.value,
                        )

            print("  static tokens:")
            for tok in tl.static_tokens:
                if tok.const_time == None:
                    print("Goal : ", tl.name, " == ", tok.value, tok.const_time)
                else:
                    print("Init : ", tl.name, " == ", tok.value, tok.const_time)

            print("--tl finished")

        print(pps.as_json(problem_tl))
        solution = pps.solve(problem_tl)
        print("-------------------")
        print("Solution:")
        print("End of time: ", solution.end_of_time)
        for tl in solution.timelines:
            for token in tl.tokens:
                print(
                    tl.name,
                    " = ",
                    token.value,
                    "[",
                    token.start_time,
                    ",",
                    token.end_time,
                    "]",
                )

        plan = self._convert_solution(solution)
        return plan
