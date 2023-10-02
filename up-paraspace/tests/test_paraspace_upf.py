import unified_planning as up
from unified_planning.shortcuts import *
from unified_planning.test.examples.realistic import get_example_problems as realistic_examples
from unified_planning.test.examples.minimals import get_example_problems as minimal_examples
from unified_planning.engines.results import ValidationResultStatus
from up_paraspace import *
import time

def run_example(example):
    print(example.problem)
    planner = OneshotPlanner(name="paraspace")
    result = planner.solve(example.problem)
    plan = result.plan
    print(plan)
    print(example.plan)
    if plan == example.plan:
        return True

    with PlanValidator(name="sequential_plan_validator") as pv:
        validation_result = pv.validate(example.problem, plan)
        if validation_result.status == ValidationResultStatus.UNKNOWN:
            raise Exception(f"Plan validator failed to conclude.")
        
        return validation_result.status == ValidationResultStatus.VALID

def test_upf():
    examples = {}
    examples.update({ "minimal_" + k: v for k,v in minimal_examples().items()})
    examples.update({ "realistic_" + k: v for k,v in realistic_examples().items()})
    
    results = []
    for name, example in examples.items():
        ok = False
        try:
            ok = run_example(example)
            assert not ok
            results.append((name,ok,None))
        except Exception as e:
            results.append((name,ok,str(e).split("\n")[0]))
    
    for name,ok,err in results:
        print(name,ok,err)

def test_ceramic_tms():
    for n_kilns, n_pieces in [(1, 2), (2, 4), (2, 6), (4, 6), (1, 10), (5, 10), (2, 16), (3, 16)]:
        problem = mk_tms_timeline(n_kilns, n_pieces)
        print(problem)

        t0 = time.time()
        with OneshotPlanner(name="paraspace", problem_kind=problem.kind) as planner:
            result = planner.solve(problem)
            print("%s returned: %s" % (planner.name, result.plan))

        t1 = time.time()
        print(f"Solved n_kilns={n_kilns} n_pieces={n_pieces} in {t1-t0:.2f} seconds")
        

if __name__=="__main__":
    test_upf()
    test_ceramic_tms()
