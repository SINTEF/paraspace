from z3 import *

all_states = ["a","b","c","d","e"]
initial = "a"
goal = "e"
transitions = [
    ("a","b"),
    ("b","d"),
    ("b","c"),
    ("d","e"),
    ("d","a")
]

s = Solver()
state_sequence = [{ initial: FreshBool() }]
s.add(state_sequence[-1][initial])

while True:
    # Try to set the last state to true
    goal_states = [selected for name, selected in state_sequence[-1].items() if name == goal]
    result = s.check(Or(goal_states))

    if result == unsat:
        print("unsat")
        
        # Add a new state
        prev_state = state_sequence[-1]
        new_state = { name: FreshBool() for name in all_states }
        
        # At least one state
        s.add(Or(list(new_state.values())))
        
        # At most one state
        for s1, b1 in new_state.items():
            for s2, b2 in new_state.items():
                if s1 != s2:
                    s.add(Or([Not(b1), Not(b2)]))
        
        # Transition relation
        for a,b in transitions:
            s.add(Implies( new_state[b], prev_state.get(a, False)))

        state_sequence.append(new_state)

    elif result == sat:
        print("success")
        model = s.model()
        for i,state in enumerate(state_sequence):
            selected = [name for name, selected in state.items() if model.eval(selected)]
            print(f"state {i}: {selected}")
        break


