Three kinds:

- **Structural**: two stages want the same hardware
- **Data**: an instruction needs a result that isn't written yet
- **Control**: the next PC is unknown until a branch resolves

Forwarding fixes most data hazards. A load-use still costs a bubble.
