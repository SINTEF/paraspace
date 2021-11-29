import json
from collections import defaultdict, namedtuple
from dataclasses import dataclass
from typing import Any


class Problem():
    def __init__(self):
        self.timelines = []
        self.tokens = []
        self.groups = defaultdict(list)

    def resource(self, classname, name=None, capacity=0):
        if name is None:
            name = f"{classname}_{len(self.tokens)}"
        self.tokens.append({ "timeline_name": name, "value": "Available",  "const_time": { "Fact": [None, None] }, "capacity": capacity, "conditions": []})
        self.groups[classname].append(name)

    def timeline(self, classname, name=None):
        if name is None:
            name = f"{classname}_{len(self.timelines)}"
        timeline = Timeline(classname, name)
        self.timelines.append(timeline)
        self.groups[classname].append(name)
        return timeline

    def goal(self, timeline, value, capacity=0):
        self.tokens.append({"timeline_name": timeline, "value": value, "const_time": { "Goal": None }, "capacity": capacity, "conditions": []})

    def fact(self, timeline, value, start=None, end=None, capacity=0):
        self.tokens.append({"timeline_name": timeline, "value": value, "const_time": { "Fact": [start,end] }, "capacity": capacity, "conditions": []})

    def to_dict(self):
        return {"groups": [{"name": key, "members": value } for key,value in self.groups.items()],
                "timelines": list(map(lambda t: t.to_dict(), self.timelines)),
                "tokens": self.tokens }

    def save_json(self,fn):
        with open(fn,"w") as f:
            json.dump(self.to_dict(), f, indent=2)
        print(f"Wrote problem instance to file {fn}")


@dataclass
class UseResource:
    resource :Any
    amount :int

@dataclass
class TransitionFrom:
    value :Any

@dataclass
class TransitionTo:
    value :Any

@dataclass
class MetBy:
    timelineref: Any
    value :Any
    amount :int = 0

@dataclass
class Meets:
    timelineref: Any
    value :Any
    amount :int = 0

@dataclass
class StartsAfter:
    timelineref: Any
    value :Any
    amount :int = 0

@dataclass
class During:
    timelineref :Any
    value :Any
    amount :int = 0

@dataclass
class Any:
    classname :Any

# UseResource = namedtuple("UseResource", "resource,amount")
# TransitionFrom = namedtuple("TransitionFrom", "value")
# MetBy = namedtuple("MetBy", "timelineref,value,amount")
# During = namedtuple("During", "timelineref,value,amount")
# Any = namedtuple("Any", "classname")

class Timeline():
    def __init__(self, classname, name=None):
        self.classname = classname
        self.name = name
        self.states = []

    def state(self, name, dur=(1,None), capacity=0, conditions=None):
        self.states.append({"name": name, "duration": dur, "capacity": capacity,
                            "conditions": list(map(lambda cond: condition_to_dict(self.name, cond), conditions or []))})

    def to_dict(self):
        return {
            "name": self.name,
            "values": self.states,
        }

def objectref_to_dict(objectref):
    if isinstance(objectref, Any):
        return {"Group": objectref.classname}
    else:
        return {"Object": objectref}

def condition_to_dict(obj_name, condition):
    if isinstance(condition, UseResource):
        return { "temporal_relationship": "Cover", "object": objectref_to_dict(condition.resource), "value": "Available", "amount": condition.amount }
    elif isinstance(condition, TransitionFrom):
        return { "temporal_relationship": "MetBy", "object": objectref_to_dict(obj_name), "value": condition.value, "amount": 0}
    elif isinstance(condition, TransitionTo):
        return { "temporal_relationship": "Meets", "object": objectref_to_dict(obj_name), "value": condition.value, "amount": 0}
    elif isinstance(condition, StartsAfter):
        return { "temporal_relationship": "StartsAfter", "object": objectref_to_dict(condition.timelineref), "value": condition.value, "amount": 0}
    elif isinstance(condition, MetBy):
        return { "temporal_relationship": "MetBy", "object":  objectref_to_dict(condition.timelineref), "value": condition.value, "amount": condition.amount }
    elif isinstance(condition, Meets):
        return { "temporal_relationship": "Meets", "object":  objectref_to_dict(condition.timelineref), "value": condition.value, "amount": condition.amount }
    elif isinstance(condition, During):
        return { "temporal_relationship": "Cover", "object":  objectref_to_dict(condition.timelineref), "value": condition.value, "amount": condition.amount }
    raise Exception(f"Unknown condition type {condition}")
