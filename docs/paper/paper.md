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
    orcid: 0000-0000-0000-0000
    equal-contrib: true 
    affiliation: 1
affiliations:
 - name: SINTEF Digital, Norway
   index: 1
date: 30 September 2023
bibliography: paper.bib

---
# Summary

In a wide variety of resarch fields such as autonomous operations and inspections in robotics, warehouse logistics and collecting data for 
natural sciences, there is a need to choose which high-level tasks to do and when, in order to obtain the system's objectives automously.
This field of research is called AI planning, which research how to solve these planning and scheduling problems.
A planning problem consist of an initial state, a desired goal state and a selection of 
available actions. For instance for a simple movable robot, the inital state and goal may be two different locations, while
possible actions maybe moving between sets of two locations.
The `paraspace` software is to solve time-based planning problems, meaning that states and actions have duration in time and the goals 
may include time deadlines. The software uses an novel algorithm  to find a solution with advantages on selected problems.
The purpose of the software is to make its capabilities available for not only for AI planning resarchers developing the field of planning,
but also for resarches in need of (task/AI) planning in diverse fields such as autonoums robotics, manufactoring, agriculture and biology.
  
# Statement of need

`paraspace` is a software for planning of temporal problems, meaning it considers time as part of the problem.
The most used (classical) planners [@ghallab1998pddl] uses a synchrounous time representation, while `paraspace`
uses an asynchrountious one, so called timeline-based. The most common way to make a planner based on timelines, is to build a 
costum constraint solver and integrate it with a selected search algorithm.  There exists a several other planner softwares based on this concept, 
such as oRatio [@de2020lifted], FAPE[@FAPE] and Europa [@barreiro2012europa], however these planner softwares have large code bases 
(12k-100k lines of code). There exist simpler implementations of planners, like LCP [@bit2018constraint] and ANML SMT [@valentini2020temporal] 
using off-the-shelf constraint solvers in contrast to costum ones such as the Z3 SMT solvers [@z3]. `paraspace` uses also the Z3 SMT solver 
combined with a novel algorithm ensuring that the search space for a solution not gets unnecarissly large. 
Using an off-the-shelf solver makes the planner software simpler, more flexible and easier to extend. 

The design of our planner opens for better performance for several planning problems such as scheduling-heavy problems. 
By scehduling-heavy it is meant problems where the timing between variables are essential. For instance an underwater multi-robot system 
with a set of given inspection points with time deadlines, meaning the inspection points needs to be inspected before the deadlines. 
The required planning between robots are minimal(yet needed).
Such a problem can be relevant for instance inspection and maintainance of equipment under water for aquaculture applications 
or collecting data for a resarch project researching life under water. A version of this problem is used in the tutorial of the software.  

The planner software itself is provided in the programmering language Rust, however as part of the software there is a python API 
for easier use. The software is available on the PyPi-platform.  
Further, the planner is also integrated into the Unified Planning Framework UPF [@upf], 
which is a python package with the goal of making planners and other planning technology easier to acess and use by minimizing the efforts to 
switch between planners and other planning technology. The integration into UPF doesn't only offer an api into the useful framework, it also include
software for converting more classical problems into timelines problems. This convertion can be useful by other AI planning researches in the pursuit 
of bridging timelines and classical problems, either theoretically or for own planner implementations. 

The software have been used as part of an underwater robotics use-case, with scientific article published here [@LUTEBERGET2022].
It have been used in an on-going research project on inspection and maintance robotics [@robplan] and integrated as a feature to the Unified Planning Framework developed 
by the EU project AIPLAN4EU [@aiplan4eu].

# Acknowledgements

We acknowledge financial support from the research project ROBPLAN [@robplan] funded by the Norwegian Research Council (RCN), grant number 322744
and the EU H2020 project AIPLAN4EU [@aiplan4eu], grant number 101016442, in order to develop this software. 

# References