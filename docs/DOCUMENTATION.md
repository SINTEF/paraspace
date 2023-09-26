# API Documentation 
The most central datatypes and functions of the paraspace planner are described in this section.

### Timeline
- Description: Datatype describing a timeline
- Variables: 
	- name: name of timeline
	- token_types: list of tokentypes for the timeline
	- static_tokens: static tokens of the timeline

### TokenType
- Description: Datatype describing a given tokentype
- Variables: 
	- value: value
	- conditions: list temporal conditions needed to hold for the token type
	- duration limits: duration limits (ex. (5,6)- meaning a duration between 5 and 6)
	- capacity: capacity of the token types to model resorces

### TemporalCond
- Description: Datatype for a temporal condition of a given timeline 
- Variables: 
	- temporal_relation: temporal relation that describes relation between two values of two timelines (MetBy, Meets, Cover and so on)
	- amount: used amount of token types capacity
	- timeline: timeline
	- value: value

### StaticToken
- Description: Datatype for one instance of tokentype
- Variables: 
	- value: value
	- const_time: time for value to hold (ex. goal,fact)
	- capacity: capacity to model resources
	- conditions: list of temporal conditions

### Problem
- Description: datatype describing a timeline-based problem
- Variables:
	- timelines: a list of timelines

### paraspace.solve()
- Description: function that solves the input function problem and returns a plan for the problem
- Input: problem (Problem)
- Output: sucess(bool), plan (Plan)

### ParaspacePlanner
- Description: class of a upf oneshootplanner engine. The UPF framework is described
- Selected Functions: 
	- compile(): compiles and grounds a problem to be suited for the planner
	- decompile(): decompiles the plan
	- solve(): converts timelines problem into UPF (classical) problem and solves it with the paraspace planner
	- supported_kind: static method giving the upf problem kind supported by the planner