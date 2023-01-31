from timelinedsl import *

for plates in [1,2]:
    for n_carbonaras in [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 15, 20, 25, 30, 40, 50, 75, 100]:

        p = Problem()

        for _ in range(plates):
            p.resource("Plate", capacity=1)

        water1 = p.timeline("Water")
        water1.state("Heating", dur=(10, 10), conditions=[UseResource(Any("Plate"), 1)])
        water1.state("HotWater", conditions=[TransitionFrom("Heating")])

        oil1 = p.timeline("Oil")
        oil1.state("Heating", dur=(10, 10), conditions=[UseResource(Any("Plate"), 1)])
        oil1.state("HotOil", conditions=[TransitionFrom("Heating")])

        for i in range(n_carbonaras):
            spaghetti = p.timeline(classname="Spaghetti", name=f"spaghetti_{i}")
            spaghetti.state("Cooking", dur=(5, 5), conditions=[During(Any("Water"), "HotWater")])
            spaghetti.state("Cooked", conditions=[TransitionFrom("Cooking")])

            lardon = p.timeline(classname="Lardon", name=f"lardon_{i}")
            lardon.state("Cooking", dur=(5, 5), conditions=[During(Any("Oil"), "HotOil")])
            lardon.state("Cooked", conditions=[TransitionFrom("Cooking")])

            eggs = p.timeline(classname="Eggs", name=f"eggs_{i}")
            eggs.state("Beating", dur=(5, 5))
            eggs.state("Beaten", conditions=[TransitionFrom("Beating")])

            carbonara = p.timeline(classname="Carbonara", name=f"carbonara_{i}")
            carbonara.state("Cooking", dur=(3, 3), conditions=[
                StartsAfter(f"spaghetti_{i}", "Cooked"),
                StartsAfter(f"lardon_{i}", "Cooked"),
                StartsAfter(f"eggs_{i}", "Beaten"),
                UseResource(Any("Plate"), 1)
            ])
            carbonara.state("Cooked", conditions=[TransitionFrom("Cooking")])
            carbonara.state("Eating", dur=(5, 5), conditions=[TransitionFrom("Cooked")])
            carbonara.state("Eaten", conditions=[TransitionFrom("Eating")])

            p.goal(f"carbonara_{i}", "Eaten")

        p.save_json(f"examples/carbonara_{plates}p_{n_carbonaras}c.json")
