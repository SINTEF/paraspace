import pyparaspace

def test_pyparaspace():
    problem = pyparaspace.Problem(
        groups=[],
        timelines=[pyparaspace.Timeline(name="obj", values=[pyparaspace.Value(name="s1", conditions=[], duration=(5,6), capacity=0), pyparaspace.Value(name="s2", conditions=[pyparaspace.Condition(temporal_relationship=pyparaspace.TemporalRelationship.MetBy, amount=0, object=["obj"], value="s1")], duration=(1,None), capacity=0)])],
        tokens=[pyparaspace.Token(timeline_name="obj", value="s2", const_time=pyparaspace.goal(), capacity=0, conditions=[])])

    solution = pyparaspace.solve(problem)
    print(f"Solution: {solution}")
    print(f"Number of tokens should be 2. Actual: {len(solution.tokens)}")
    token1 = solution.tokens[1]
    token2 = solution.tokens[0]
    print(f"Token value 1 should be s1. Actual value: {token1.value}")
    print(f"Token value 2 should be s2. Actual value: {token2.value}")
    print(f"Should be true. Actual value: {token1.end_time - token1.start_time >= 5. and token1.end_time - token1.start_time <= 6.}")
    print(f"Should be true. Actual value: {abs(token1.end_time - token2.start_time) < 1e-5}")
    #assert!(token2.end_time.is_infinite());

