import unified_planning as up
from unified_planning.shortcuts import *
from unified_planning.test.examples.realistic import get_example_problems as realistic_examples
from unified_planning.test.examples.minimals import get_example_problems as minimal_examples
from unified_planning.engines.results import ValidationResultStatus

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

if __name__=="__main__":
    test_upf()
