import sys, json
import matplotlib
import matplotlib.pyplot as plt
from collections import defaultdict

colors = ["red","green","blue","purple","orange","black","gray"]

filename = sys.argv[1]
print(f"Reading file {filename}")

with open(filename, "r") as f:
    model = json.load(f)

print(f"got model {model}")

fig, ax = plt.subplots(figsize=(10,6),dpi=120)

def rat(x):
    return float(x[0]) / float(x[1])

y_stack = []
y_names = {}

timelines = defaultdict(list)
for token in model["tokens"]:
    timelines[token["object_name"]].append(token)

for (timeline_name,tokens) in timelines.items():
    y_names[timeline_name] = len(y_stack)
    intervals = []
    for token in tokens:
        start = token["start_time"] if token["start_time"] is not None else  token["end_time"] - 1
        end = token["end_time"] if token["end_time"] is not None else  token["start_time"] + 1
        intervals.append((start, end, colors[hash(token["value"]) % len(colors)], token["value"]))
    y_stack.append( { "name": timeline_name, "intervals" : intervals })

#y_names["visibility"] = len(y_stack)
#y_stack.append({ "name": "visibility", "intervals": [(rat(x[0]),rat(x[1]),"red") for x in model["problem"]["visibility_time_windows"]] })
#
#for (l_idx, location) in enumerate(model["locations"]):
#    name = f"loc{l_idx}_@{location[0]}"
#    y_names[name] = len(y_stack)
#    y_stack.append({"name": name, "intervals": [(location[1][0], location[1][1], "blue")]})
#
#for (_idx, take_picture) in enumerate(model["take_picture"]):
#    name = f"take_@{take_picture[0]}"
#    y_names[name] = len(y_stack)
#    y_stack.append({"name": name, "intervals": [(take_picture[1][0], take_picture[1][1], "green")]})
#
#for (_idx, download) in enumerate(model["download"]):
#    name = f"dl_{download[0]}"
#    y_names[name] = len(y_stack)
#    y_stack.append({"name": name, "intervals": [(download[1][0], download[1][1], "orange")]})

#for (idx,water) in enumerate(model["problem"]["visibility_time_windows"]):
#    name = f"water{idx}"
#for (idx,oil) in enumerate(model["oil_heat"]):
#    name = f"oil{idx}"
#    y_names[name] = len(y_stack)
#    y_stack.append({ "name": name, "intervals": [(oil[1],oil[2],"red")] })
#
#for (idx,carbonara) in enumerate(model["carbonaras"]):
#    name = f"carbonara{idx}"
#    y_names[name] = len(y_stack)
#    y_stack.append({ "name": name, "intervals": [
#        (carbonara["spaghetti_start"],carbonara["spaghetti_end"],"red"),
#        (carbonara["lardon_start"],carbonara["lardon_end"],"green"),
#        (carbonara["eggs_start"],carbonara["eggs_end"],"blue"),
#        (carbonara["cook_start"],carbonara["cook_end"],"orange"),
#        (carbonara["eat_start"],carbonara["eat_end"],"purple"),
#    ]})


ax.set_ylim([-0.5, len(y_stack)-1 +0.5])
ax.set_yticks(list(range(len(y_stack))))
ax.set_yticklabels([x["name"] for x in y_stack])
fig.subplots_adjust(left=0.3)

for item in y_stack:
    y = y_names[item["name"]]
    print(f"item {item}")
    for i,x in enumerate(item["intervals"]):
        ax.broken_barh([(x[0],x[1]-x[0])], 
                       (y-0.4+0.1*i,0.1),
                       facecolor=[x[2]])
        ax.text(x[0], y-0.4+0.1*i+0.05, x[3])

# ARROWS
#for (structure_idx,structure) in enumerate(model["problem"]["structures"]):
#    for p_idx in [structure["piece_1"], structure["piece_2"]]:
#        p_x = model["pieces"][p_idx]["treated"]
#        p_y = y_names[f"piece{p_idx}"]
#        s_x = model["structures"][structure_idx]["assemble"]
#        s_y = y_names[f"structure{structure_idx}"]
#        ax.arrow(p_x, p_y, s_x - p_x, s_y - p_y, color="blue", head_width=0.1, head_length=0.15, length_includes_head=True)


plt.savefig(f"{filename}.plot.png")
