from timelinedsl import *

for (n_kilns,n_pieces) in [(1,2),(2,4),(2,6),(4,6),(5,10)]:

    p = Problem()
    p.resource("Electricity", capacity=1)

    kiln_capacities = []
    for kiln_idx in range(n_kilns):
        kiln = p.timeline("Kiln", f"kiln_{kiln_idx}")
        kiln.state("Ready", conditions=[TransitionFrom("Fire")])
        kiln.state("Fire", dur=(20,20), conditions=[
            TransitionFrom("Ready"), 
            UseResource(Any("Electricity"), 1)])
        kiln_capacities.append(p.resource("KilnSpace", capacity=2))
        p.fact(f"kiln_{kiln_idx}", "Ready")

    piece_param_types = [(5,2),(8,3),(11,1)]
    pieces = []
    while len(pieces) < n_pieces:
        pieces.append(piece_param_types[len(pieces) % len(piece_param_types)])

    for (piece_idx,(bake_time,treat_time)) in enumerate(pieces):
        piece = p.timeline("Piece", f"piece_{piece_idx}")
        piece.state("Baking", dur=(bake_time,bake_time), conditions=[
            UseResource(Any("KilnSpace"), 1),
            During(Any("Kiln"), "Fire"),
        ])
        piece.state("Baked", conditions=[TransitionFrom("Baking")])
        piece.state("Treating", dur=(treat_time, treat_time), 
            conditions=[TransitionFrom("Baked")])
        piece.state("Treated", conditions=[TransitionFrom("Treating")])

        if piece_idx >= 2*(n_pieces//2):
            p.goal(f"piece_{piece_idx}", "Baked")

    for structure_idx in range(n_pieces // 2):
        structure = p.timeline("Structure", f"structure_{structure_idx}")
        structure.state("Assembling", dur=(1,1), conditions=[
            MetBy(f"piece_{2*structure_idx}", "Treated"),
            MetBy(f"piece_{2*structure_idx +1}", "Treated")
        ])
        structure.state("Assembled", conditions=[TransitionFrom("Assembling")])
        structure.state("Baking", dur=(3,3), conditions=[
            UseResource(Any("KilnSpace"), 1),
            During(Any("Kiln"), "Fire"),
            TransitionFrom("Assembled"),
        ])
        structure.state("Baked", conditions=[TransitionFrom("Baking")])
        p.goal(f"structure_{structure_idx}", "Baked")

    p.save_json(f"examples/ceramic_{n_kilns}m_{n_pieces}j.json")
