x86 is TSO: a store can be delayed past a later load, and that's about it. ARM is weaker, so code that worked on my laptop breaks on an ARM server. Use acquire/release, not hope.
