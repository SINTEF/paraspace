---
title: 'ParaSpace : A timeline-based Temporal Planning Software'
tags:
  - python
  - planning
  - temporal
authors:
  - name: Bjørnar Luteberget
    orcid: 0000-0000-0000-0000
    equal-contrib: true
    affiliation: 1 
  - name: Eirik Kjeken
    orcid: 0000-0000-0000-0000 
    equal-contrib: true 
    affiliation: 1
  - name: Synne Fossøy
    orcid: 0000-0003-4139-5486
    equal-contrib: true 
    affiliation: 1
affiliations:
 - name: SINTEF Digital, Norway
   index: 1
date: 30 November 2023
bibliography: paper.bib

---
# Summary

Autonomous robots (ie.., unmanned vehicles and robot manipulators) can contribute in a wide variety of applications such 
as inspection and maintenance, drones in search and rescue, warehouse logistics and collecting data for natural sciences. 
But to enable full autonomy, the robots need to be able to plan high-level tasks themselves.    
Such high-level tasks may for instance be moving between locations, handling and lifting objects, or more complex tasks such as searching for an object.  
This field of research is called automated planning and focuses on how to solve these planning and scheduling problems and how to execute computed plans.
A classical planning problem consists of an initial state, a desired goal state, and a selection of available actions. 
For instance, for an unmanned ground vehicle (UGV), the initial state and goal may be two different locations, while
possible actions may be moving between locations.
We introduce in this paper a software for automated planning called `paraspace`. 
The software solves time-based planning problems, meaning that states and actions have duration in time (e.g., the time for an UGV to move between two locations) and the goals 
may include time deadlines (e.g., the UGV should reach a location before given time). The software uses a novel algorithm to find a solution advantageous for selected problems. 
The software is both available as a stand-alone software package and as a part of the Unified Planning Framework (UPF) [@upf].
The purpose of publishing this software is to make its capabilities available not only for AI planning researchers developing the field of planning,
but also for researchers in need of automated planning working with enabling autonoumous techonolgy in an variety of application areas.
  
# Statement of need

`paraspace` is software for planning temporal problems, meaning it considers time as part of the problem.
The most used (classical) planners [@ghallab1998pddl] use a synchronous time representation where the whole state space is equally discretized in time. 
On the other hand,  `paraspace` uses an asynchronous one, where the state space is discretized differently for each variable.
This is called timeline-based planning. The most common way to make a planner based on timelines is to build a 
custom constraint solver and integrate it with a selected search algorithm.  There exist several other planner software based on this concept, 
such as oRatio [@de2020lifted], FAPE[@FAPE], and Europa [@barreiro2012europa], however, these planner software have large code bases 
(12k-100k lines of code). There exist simpler implementations of planners, like LCP [@bit2018constraint] and ANML SMT [@valentini2020temporal] 
using off-the-shelf constraint solvers in contrast to custom ones such as the Z3 SMT solvers [@z3].
Using an off-the-shelf solver makes the planner software simpler, more flexible, and easier to extend. `paraspace` uses also the Z3 SMT solver 
combined with a novel algorithm ensuring that the search space for a solution does not get unnecessarily large. 

The design of our planner opens for better performance for several planning problems such as scheduling-heavy problems. 
Scheduling-heavy problems are problems where the timing between variables is essential. For instance an underwater multi-robot system 
with a set of given inspection points with time deadlines, meaning the inspection points need to be inspected before the deadlines. 
The required planning between robots is minimal, yet needed.
Such a problem can be relevant for instance inspection and maintenance of equipment underwater for aquaculture applications 
or collecting data for a research project researching life under water. A version of this problem is used in the tutorial of the software.  

The planner software itself is provided in the programming language Rust, however, it is equipped with a Python API 
for easier use. The software is available on the PyPi platform.  
Further, the planner is also integrated into the Unified Planning Framework UPF [@upf], 
which is a Python-based framework for AI planning.
The objectives of UPF are to make planners and other planning technology easily accessible by minimizing the efforts needed
to get started with planning for beginners and to switch between planners and other planning technologies for more experienced users.
The integration into UPF does not only offer an API into the useful framework, but it also includes 
software for converting more classical problems into timelines problems. This conversion can be useful by other AI planning researchers in the pursuit 
of bridging timelines and classical problems, either theoretically or for their planner implementations. 

The software has been used as part of an underwater robotics use case, with a scientific article published here [@LUTEBERGET2022].
It has been used in an ongoing research project on inspection and maintenance robotics [@robplan] and integrated as a feature of the Unified Planning Framework developed 
by the EU project AIPLAN4EU [@aiplan4eu].

# Acknowledgements

We acknowledge financial support from the research project ROBPLAN [@robplan] funded by the Norwegian Research Council (RCN), grant number 322744
and the EU H2020 project AIPLAN4EU [@aiplan4eu], grant number 101016442, to develop this software. 

# References