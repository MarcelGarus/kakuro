# Kakuro solver

Kakuros are cool puzzles.
This repo contains several solvers with different strategies.

## Measurements

I conducted ten measurements for each value with prior warm-up.

oom = Out of memory and killed by the operating system  
timeout = Took longer than 30 minutes

solver                | small     | wikipedia | 15x15     | 20x20     | 30x30      | book       |
----------------------|-----------|-----------|-----------|-----------|------------|------------|
naive                 |  18.29 s  | timeout   | TODO      | TODO      | TODO       | TODO       |
gradual               |   4.79 ms |   5.17 s  | 599.65 s  | TODO      | TODO       | timeout    |
sum_reachable         |   2.31 ms |  39.96 ms | 148.44 ms |   1.57 s  | 101.653 s  | 137.72 s   |
prioritize            |   1.39 ms | 434.67 ms | 543.49 ms | 400.71 s  | timeout    | 695.64 s   |
sum_reachable_no_set  | 691.78 us |  10.64 ms |  51.78 ms | 449.40 ms |   45.45 s  |  38.98 s   |
divide                |  37.40 us |   2.45 ms |  12.51 ms | 121.02 ms |    5.22 s  | oom        |
connecting_cells      |  47.37 us |   1.77 ms |  12.30 ms |  34.40 ms |  550.77 ms | oom        |
lazy                  |  65.39 us |   2.61 ms |  12.78 ms |  86.40 ms |   11.14 s  | oom        |
propagate_constraints | 188.25 us |   1.64 ms |  10.25 ms |  43.84 ms | oom        |   1.42 s   |
only_check_changes    | 201.31 us |   1.35 ms |   5.57 ms |  36.28 ms |    4.22 s  |   3.62 s   |
solution_in_rc        | 104.24 us |   1.06 ms |   8.43 ms |  38.05 ms | oom        | 443.53 ms  |

## Raw measurements

The values are median with standard deviation, as well as minimum and maximum.

- naive
  - small: 18.293 s +- 1.23%; 17.901 - 18.543 s
  - wikipedia: >35 min
- gradual
  - small: 4.790 ms +- 0.36%;  4.756 -  4.806 ms
  - wikipedia: 5.174 s +- 0.76%; 5.128 - 5.244 s
  - 15x15: 599.651 s +- 0.72%; 596.472 - 604.706 s
  - book: >35 min
- sum_reachable
  - small: 2.310 ms +- 0.30%; 2.295 - 2.317 ms
  - wikipedia: 39.968 ms +- 0.23%; 39.747 - 401.241 ms
  - 15x15: 148.437 ms +- 0.27%; 147.605 - 148.950 ms
  - book: 137.720 s +- 0.69%; 136.126 - 138.796 s
  - 20x20: 1.570 s +- 0.34%; 1.564 - 1.580 s
  - 30x30: 101.653 s +- 0.60%; 100.608 - 102.538 s
- prioritize
  - small: 1.385 ms +- 0.42%; 1.375 - 1.394 ms
  - wikipedia: 434.679 ms +- 0.48%; 431.497 - 437.421 ms
  - 15x15: 543.487 ms +- 0.27%; 540.722 - 545.361 ms
  - book: 695.638 s +- 0.78%; 689.53 - 704.844 s
  - 20x20: 400.710 s +- 0.62%; 394.727 - 403.980 s
  - 30x30: >42 min
- sum_reachable_no_set
  - small: 691.776 us +- 0.47%; 687.491 - 697.260 us
  - wikipedia: 10.638 ms +- 0.29%; 10.563 - 10.669 ms
  - 15x15: 51.776 ms +- 0.13%; 51.648 - 51.896 ms
  - 20x20: 449.398 ms +- 0.51%; 446.849 - 453.351 ms
  - 30x30: 45.448 s +- 2.97%; 44.173 - 47.950 s
  - book: 38.977 s +- 0.32%; 38.783 - 39.163 s
- divide
  - small: 37.397 us +- 5.15%; 36.338 - 42.941 us
  - wikipedia: 2.452 ms +- 10.78%; 2.275 - 2.072 ms
  - 15x15: 12.510 ms +- 0.49%; 12.446 - 12.473 ms
  - 20x20: 121.024 ms +- 0.62%; 120.278 - 123.050 ms
  - 30x30: 5.218 s +- 0.88%; 5.128 - 2.268 s
  - book: OOM after 13 min, 31.2 GiB -> 12.7 GiB
- connecting_cells
  - small: 47.369 us +- 3.46%; 46.307 - 51.948 ns
  - wikipedia: 1.770 ms +- 0.69%; 1.753 - 1.791 ms
  - 15x15: 12.301 ms +- 5.85%; 10.064 - 12.108 ms
  - 20x20: 34.403 ms +- 1.74%; 30.029 - 36.112 ms
  - 30x30: 550.773 ms +- 1.11%; 543.086 - 565.360 ms
  - book: OOM after 1 min, 31.2 GiB -> 12.7 GiB
- lazy
  - small: 65.388 us +- 5.01%; 62.989 - 73.880 us
  - wikipedia: 2.608 ms +- 0.93%; 2.588 - 2.677 ms
  - 15x15: 12.783 ms +- 0.65%; 12.718 - 13.026 ms
  - 20x20: 86.406 ms +- 2.14%; 83.346 - 89.416 ms
  - 30x30: 11.138 s +- 1.66%; 10.876 - 11.446 s
  - book: OOM after 1.5 min, 31.2 GiB -> 12.7 GiB
- propagate_constraints
  - small: 188.253 us +- 3.26%; 18.343 - 20.218 us
  - wikipedia: 1.638 ms +- 4.86%; 1.584 - 1.837 ms
  - 15x15: 10.246 ms +- 0.21%; 12210 - 10.289 ms
  - 20x20: 43.843 ms +- 0.33%; 43.660 - 44.101 ms
  - 30x30: OOM after 30s
  - book: 1.415 s +- 0.76%; 1.400 - 1.432 s
- only_check_changes
  - small: 201.306 us +- 0.69%; 200.287 - 205.166 us
  - wikipedia: 1.348 ms +- 0.42%; 1.341 - 1.357 ms
  - 15x15: 5.573 ms +- 0.15%; 5.560 - 5.586 ms
  - 20x20: 36.277 ms +- 0.22%; 36.198 - 36.460 ms
  - 30x30: 4.222 s +- 1.07%; 4.132 - 4.290 s
  - book: 3.623 s +- 0.72%; 3.587 - 3.660 s
- solution_in_rc
  - small: 104.234 us +- 3.64%; 99.479 - 113.033 us
  - wikipedia: 1.060 ms +- 1.42%; 1.042 - 1.092 ms
  - 15x15: 8.434 ms +- 1.42%; 8.277 - 8.68 ms
  - 20x20: 38.052 ms +- 0.87%; 37.440 - 38.596 ms
  - 30x30: oom
  - book: 443.534 ms +- 2.62%; 432.134 - 465.234 ms

## Todo

- invent new solvers
