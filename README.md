# Kakuro solver

Kakuros are cool puzzles.
This repo contains several solvers with different strategies.

## Measurements

algo        | micro | mini |  small | wikipedia | medium | book
------------|-------|------|--------|-----------|--------|-------
naive       |   1ms |  1ms | 3542ms |     >300s |  >200s |  >200s
gradual     |   1ms |  2ms |   21ms |    9924ms |  >360s |      -
early_abort |   1ms |  1ms |    6ms |      50ms |  157ms |   135s
prioritize  |   1ms |  1ms |    4ms |     387ms |  490ms |  >120s
divide      |   1ms |  1ms |    1ms |       3ms |   24ms | OOM after 38s
lazy        |   1ms |  1ms |    1ms |       9ms |   60ms | OOM after 230s
connections |   1ms |  1ms |    5ms |       5ms |   15ms | 2021ms
