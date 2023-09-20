import math
import unittest
import pyparaspace


def test_pyparaspace():
    problem = pyparaspace.Problem(
        timelines=[
            pyparaspace.Timeline(
                name="obj",
                token_types=[
                    pyparaspace.TokenType(
                        value="s1", conditions=[], duration_limits=(5, 6), capacity=0
                    ),
                    pyparaspace.TokenType(
                        value="s2",
                        conditions=[
                            pyparaspace.TemporalCond(
                                temporal_relation=pyparaspace.TemporalRelation.MetBy,
                                amount=0,
                                timeline="obj",
                                value="s1",
                            )
                        ],
                        duration_limits=(1, None),
                        capacity=0,
                    ),
                ],
                static_tokens=[
                    pyparaspace.StaticToken(
                        value="s2",
                        const_time=pyparaspace.goal(),
                        capacity=0,
                        conditions=[],
                    )
                ],
            )
        ],
    )

    solution = pyparaspace.solve(problem)
    print(f"Solution: {solution}")
    timeline = solution.timelines[0]
    assert len(timeline.tokens) == 2, "Number of tokens should be 2"

    token1 = timeline.tokens[0]
    token2 = timeline.tokens[1]
    assert token1.value == "s1", "Token value 1 should be s1"
    assert token2.value == "s2", "Token value 2 should be s2"

    assert (
        token1.end_time - token1.start_time >= 5.0
        and token1.end_time - token1.start_time <= 6.0
    )
    assert abs(token1.end_time - token2.start_time) < 1e-5


if __name__ == "__main__":
    test_pyparaspace()
