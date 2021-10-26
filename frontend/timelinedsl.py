import json
from collections import namedtuple

class Problem():
    def __init__(self):
        self.resources = []
        self.timelines = []
        self.facts = []
        self.goals = []

    def resource(self, classname, name=None, capacity=None):
        if name is None:
            name = f"{classname}_{len(self.resources)}"
        resource = {"class": classname, "name": name, "capacity": capacity}
        self.resources.append(resource)
        return resource

    def timeline(self, classname, name=None):
        timeline = Timeline(classname, name)
        self.timelines.append(timeline)
        return timeline

    def goal(self, timeline, value):
        self.goals.append({"timeline_name": timeline, "value": value})

    def fact(self, timeline, value):
        self.facts.append({"timeline_name": timeline, "value": value})

    def to_dict(self):
        return {"resources": self.resources,
                "timelines": list(map(lambda t: t[1].to_dict(t[0]), enumerate(self.timelines))),
                "facts": self.facts,
                "goals": self.goals}

    def save_json(self,fn):
        with open(fn,"w") as f:
            json.dump(self.to_dict(), f, indent=2)
        print(f"Wrote problem instance to file {fn}")


UseResource = namedtuple("UseResource", "resource,amount")
TransitionFrom = namedtuple("TransitionFrom", "value")
MetBy = namedtuple("MetBy", "timelineref,value")
During = namedtuple("During", "timelineref,value")
Any = namedtuple("Any", "classname")
ProvideResource = namedtuple("ProvideResource", "classname,capacity")

class Timeline():
    def __init__(self, classname, name=None):
        self.classname = classname
        self.name = name
        self.states = []

    def state(self, name, dur=(1,None), conditions=None):
        self.states.append({"name": name, "duration": dur,
                            "conditions": list(map(condition_to_dict, conditions or []))})

    def to_dict(self, idx):
        return {
            "class": self.classname,
            "name": self.name or f"{self.classname}_{idx}",
            "states": self.states,
        }

def objectref_to_dict(objectref):
    if isinstance(objectref, Any):
        return {"AnyOfClass": objectref.classname}
    else:
        return {"Named": objectref}

def timelineref_to_dict(timelineref):
    return { "timeline_name": timelineref }

def condition_to_dict(condition):
    if isinstance(condition, UseResource):
        return { "UseResource": [ objectref_to_dict(condition.resource), condition.amount ]}
    elif isinstance(condition, TransitionFrom):
        return { "TransitionFrom": condition.value}
    elif isinstance(condition, MetBy):
        return { "MetBy": [ objectref_to_dict(condition.timelineref), condition.value ]}
    elif isinstance(condition, During):
        return { "During": [ objectref_to_dict(condition.timelineref), condition.value ]}
    elif isinstance(condition, ProvideResource):
        return { "ProvideResource": [ condition.classname, condition.capacity ]}
    raise Exception(f"Unknown condition type {condition}")
