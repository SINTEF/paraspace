from z3 import *

conditions = {
    "water.cold": [],
    "water.heating": ["water.cold"],
    "water.hot": ["water.heating"],
    "spaghetti.uncooked": [],
    "spaghetti.cooking": ["spaghetti.uncooked"],
    "spaghetti.cooked": ["spaghetti.cooking"],
}

goal = "spaghetti.cooked"

def solve(conditions, goal):
    s = Solver()
    tokens = []
    tokens_by_name = {}

    # Add goal token
    tokens.append({
        "name": goal,
        "active": FreshBool(),
        "start": FreshReal(),
        "end": FreshReal(),
    })
    tokens_by_name[goal] = [tokens[0]]

    extension_literals = {}

    # Add conditions for goal token
    for condition in conditions[goal]:
        new_tokens_var = FreshBool(f"new_{condition}")

        condition_satisfied = [ 
            And(t["active"], t["end"] <= tokens[0]["start"])
            for t in tokens if t["name"] == condition]

        s.add(Implies(tokens[0]["active"], 
            Or(Or( condition_satisfied ), new_tokens_var )))

        extension_literals[Not(new_tokens_var)] = condition
        
    s.add(tokens[0]["active"])

    while True:
        result = s.check(list(extension_literals.keys()))
        if result == unsat:
            # Create new tokens
            print("UNSAT")
            core = s.unsat_core()
            print(core)
            pass
        elif result == sat:
            return True


solve(conditions, goal)