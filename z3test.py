from z3 import *

s = Solver()

x = FreshInt()
y = FreshInt()

s.add(x > 5)

print(s.check())
m = s.model()

print("x =", m.eval(x, model_completion=True))
print("y =", m.eval(y, model_completion=True))


z = FreshInt()
s.add(z > x)
s.add(y > x)
print(s.check())
m = s.model()

print("x =", m.eval(x, model_completion=True))
print("y =", m.eval(y, model_completion=True))
print("z =", m.eval(z, model_completion=True))
