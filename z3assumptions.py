from z3 import *

s = Solver()

p1 = Bool("p1")
p2 = Bool("p2")
p3 = Bool("p3")

x = Int("x")

s.add(x < 10)
s.add(x > -10)
s.add(Implies(p1, x > 0))
s.add(Implies(p2, x > 7))
s.add(Implies(p3, x < 5))


print(s.check([p1,p2,p3]))
print(s.unsat_core())

print(s.check([p1,p3]))
print(s.unsat_core())



# a, b, c, d, e, f, g

#
# a,b,c,d
#
# a, x1
# !x1, b, x2
# !x2, c, x3
# !x3, d
#
##

a = Bool("a")
b = Bool("b")
extend1 = FreshBool()
s.add(Or([a,b,extend1]))

s.check([Not(extend1)])

c = Bool("c")
extend2 = FreshBool()
s.add(Or([Not(extend1), c, extend2 ]))


s.check([Not(extend2)])