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
from .classical_conversion import ParaspaceClassicalProblemConversion
from .timelines_conversion import ParaspaceTimelinesProblemConversion

class ParaspacePlanner(engines.Engine, engines.mixins.OneshotPlannerMixin):
    def __init__(self, **options):
        # Read known user-options and store them for using in the `solve` method
        engines.Engine.__init__(self)
        engines.mixins.OneshotPlannerMixin.__init__(self)
        self.compiler_res = []
        self.compilations = [CompilationKind.CONDITIONAL_EFFECTS_REMOVING,
                             CompilationKind.GROUNDING]
    @property
    def name(self) -> str:
        return "paraspace"

    @staticmethod
    def supported_kind():
        supported_kind = ProblemKind()
        supported_kind.set_problem_class('ACTION_BASED')
        supported_kind.set_typing('FLAT_TYPING')
        supported_kind.set_quality_metrics
        supported_kind.set_fluents_type('OBJECT_FLUENTS')
        supported_kind.set_conditions_kind('EQUALITIES')
        supported_kind.set_conditions_kind('NEGATIVE_CONDITIONS')
        supported_kind.set_quality_metrics("OVERSUBSCRIPTION")

        supported_kind.set_time('CONTINUOUS_TIME')
        supported_kind.set_expression_duration('STATIC_FLUENTS_IN_DURATIONS')
        return supported_kind

    @staticmethod
    def supports(problem_kind):
        return problem_kind <= ParaspacePlanner.supported_kind()
    
    def _compile(self,problem:"up.model.Problem"):
        prob = problem
        self.compiler_res = []
        for compKind in self.compilations:
            # Check if you need to do a compilation before hand !! :)
            try:
                with Compiler(problem_kind=prob.kind,compilation_kind=compKind) as compiler:
                    res = compiler.compile(prob,compKind)
                    prob = res.problem
                    self.compiler_res += [res]
            except Exception as e:
                print(e)
        return prob
    
    def _solve(
        self,
        problem: "up.model.Problem",
        callback: Optional[Callable[["up.engines.PlanGenerationResult"], None]] = None,
        timeout: Optional[float] = None,
        output_stream: Optional[IO[str]] = None,
    ) -> "up.engines.results.PlanGenerationResult":
        original_problem = problem
        env = problem.environment

        problem = self._compile(problem)
        # if not self.supports(problem.kind):
        #     raise Exception("Problemkind not supported" + str(problem.kind))
        
        problem_converters = [
            ParaspaceTimelinesProblemConversion,
            ParaspaceClassicalProblemConversion,
        ]

        for conv_class in problem_converters:
            try:
                conv = conv_class(problem, self.compiler_res)
                plan = conv.convert_and_solve(problem)

                status = (
                    PlanGenerationResultStatus.UNSOLVABLE_INCOMPLETELY
                    if plan is None
                    else PlanGenerationResultStatus.SOLVED_SATISFICING
                )
                return up.engines.PlanGenerationResult(status, plan, self.name)
            except ParaspaceTimelinesPlannerConversionError as e:
                pass
        
        raise ParaspaceTimelinesPlannerConversionError("No converters succeeded in translating the problem to paraspace.")
    
env = up.environment.get_environment()
env.factory.add_engine('paraspace', __name__, 'ParaspacePlanner')