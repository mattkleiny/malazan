# Game

This is the project's ubiquitous language: one line per term, grouped by section. Each section links to its full entries in `docs/context/` once it has them.

## Scales

- **Fixed Tick**: One deterministic step of the simulation. _Avoid_: frame, update
- **Variable Tick**: One presentation step, taken once per rendered frame.
- **Simulation Clock**: The clock the simulation keeps for itself, handing out fixed ticks as time passes. _Avoid_: timer, physics rate
