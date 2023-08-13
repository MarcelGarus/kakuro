# Kakuro solver

Kakuros are cool puzzles.
This repo contains several solvers with different strategies.

## Measurements

I conducted ten measurements for each value with prior warm-up.

todo = Not measured yet  
oom = Out of memory and killed by the operating system  
timeout = Took longer than 30 minutes

| solver                   | small     | wikipedia | 15x15     | 20x20     | 30x30     | book      |
| ------------------------ | --------- | --------- | --------- | --------- | --------- | --------- |
| naive                    | todo      | todo      | todo      | todo      | todo      | todo      |
| gradual                  | todo      | todo      | todo      | todo      | todo      | todo      |
| sum_reachable            | todo      | todo      | todo      | todo      | todo      | todo      |
| prioritize               | 1.23 ms   | 376.48 ms | 466.87 ms | 364.07 s  | timeout   | todo      |
| sum_reachable_no_set     | 335.41 us | 5.12 ms   | 20.43 ms  | 208.78 ms | 14.06 s   | 20.56 s   |
| only_check_changes       | 170.99 us | 792.26 us | 2.18 ms   | 9.70 ms   | 595.41 ms | 1.41 s    |
| divide                   | 23.48 us  | 1.55 ms   | 8.13 ms   | 143.76 ms | 6.24 s    | oom       |
| connecting_cells         | 29.79 us  | 833.63 us | 4.62 ms   | 18.25 ms  | 313.25 ms | oom       |
| lazy                     | 46.41 us  | 1.47 ms   | 7.63 ms   | 68.32 ms  | 10.46 s   | oom       |
| propagate_constraints    | 155.90 us | 1.34 ms   | 8.05 ms   | 37.12 ms  | oom       | 1.12 s    |
| solution_in_rc           | 61.72 us  | 778.51 us | 6.57 ms   | 33.40 ms  | oom       | 193.86 ms |
| simpler_recursion_anchor | 45.99 us  | 681.29 us | 5.68 ms   | 28.64 ms  | oom       | 178.51 ms |
| fxhashmap                | 39.12 us  | 641.78 us | 5.35 ms   | 27.49 ms  | oom       | 142.74 ms |
| better_vecs              | 34.61 us  | 597.71 us | 5.75 ms   | 32.85 ms  | oom       | 88.97 ms  |

## Raw measurements

The values are median with standard deviation, as well as minimum and maximum.

- naive
  - small: todo
  - wikipedia: todo
  - 15x15: todo
  - 20x20: todo
  - 30x30: todo
  - book: todo
- gradual
  - small: todo
  - wikipedia: todo
  - 15x15: todo
  - 20x20: todo
  - 30x30: todo
  - book: todo
- sum_reachable
  - small: todo
  - wikipedia: todo
  - 15x15: todo
  - 20x20: todo
  - 30x30: todo
  - book: todo
- prioritize
  - small: 1.23 ms +- 0.64 %; 1.22 ms – 1.24 ms
  - wikipedia: 376.48 ms +- 0.23 %; 374.88 ms – 377.45 ms
  - 15x15: 466.87 ms +- 0.11 %; 465.69 ms – 467.73 ms
  - 20x20: 364.07 s +- 0.50 %; 360.01 s – 366.62 s
  - 30x30: >35 min
  - book: todo
- sum_reachable_no_set
  - small: 335.41 us +- 0.49 %; 333.95 us – 339.22 us
  - wikipedia: 5.12 ms +- 0.17 %; 5.10 ms – 5.13 ms
  - 15x15: 20.43 ms +- 2.91 %; 20.16 ms – 22.17 ms
  - 20x20: 208.78 ms +- 0.19 %; 208.28 ms – 209.42 ms
  - 30x30: 14.06 s +- 0.69 %; 13.98 s – 14.22 s
  - book: 20.56 s +- 1.00 %; 20.12 s - 20.79 s
- only_check_changes
  - small: 170.99 us +- 0.78 %; 169.85 us - 173.50 us
  - wikipedia: 792.26 us +- 10.94 %; 758.90 us - 1.05 ms
  - 15x15: 2.18 ms +- 4.99 %; 2.11 ms - 2.43 ms
  - 20x20: 9.70 ms +- 1.28 %; 9.55 ms - 9.96 ms
  - 30x30: 595.41 ms +- 0.84 %; 588.83 ms - 608.70 ms
  - book: 1.41 s +- 1.11 %; 1.38 s - 1.42 s
- divide
  - small: 23.48 us +- 8.30 %; 22.31 us – 28.99 us
  - wikipedia: 1.55 ms +- 0.84 %; 1.53 ms – 1.58 ms
  - 15x15: 8.13 ms +- 0.34 %; 8.10 ms – 8.18 ms
  - 20x20: 143.76 ms +- 1.01 %; 142.05 ms – 147.75 ms
  - 30x30: 6.24 s +- 1.02 %; 6.18 s – 6.36 s
  - book: oom
- connecting_cells
  - small: 29.79 us +- 6.25 %; 28.69 us – 34.81 us
  - wikipedia: 833.63 us +- 0.84 %; 822.23 us – 846.14 us
  - 15x15: 4.62 ms +- 0.53 %; 4.59 ms – 4.66 ms
  - 20x20: 18.25 ms +- 0.29 %; 18.19 ms – 18.38 ms
  - 30x30: 313.25 ms +- 0.85 %; 309.63 ms – 317.18 ms
  - book: oom
- lazy
  - small: 46.41 us +- 6.93 %; 43.21 us – 54.47 us
  - wikipedia: 1.47 ms +- 1.40 %; 1.45 ms – 1.52 ms
  - 15x15: 7.63 ms +- 1.29 %; 7.49 ms – 7.87 ms
  - 20x20: 68.32 ms +- 4.33 %; 64.57 ms – 74.80 ms
  - 30x30: 10.46 s +- 1.89 %; 9.96 s – 10.70 s
  - book: oom
- propagate_constraints
  - small: 155.90 us +- 3.30 %; 151.01 us – 166.42 us
  - wikipedia: 1.34 ms +- 3.23 %; 1.31 ms – 1.47 ms
  - 15x15: 8.05 ms +- 0.56 %; 7.99 ms – 8.14 ms
  - 20x20: 37.12 ms +- 2.45 %; 36.11 ms – 38.90 ms
  - 30x30: oom
  - book: 1.12 s +- 0.79 %; 1.10 s – 1.13 s
- solution_in_rc
  - small: 61.72 us +- 6.56 %; 56.98 us – 71.14 us
  - wikipedia: 778.51 us +- 1.55 %; 766.99 us – 803.07 us
  - 15x15: 6.57 ms +- 0.27 %; 6.53 ms – 6.60 ms
  - 20x20: 33.40 ms +- 0.69 %; 33.12 ms – 33.77 ms
  - 30x30: oom
  - book: 193.86 ms +- 4.22 %; 186.47 ms - 210.47 ms
- simpler_recursion_anchor
  - small: 45.99 us +- 7.82 %; 42.07 us – 55.11 us
  - wikipedia: 681.29 us +- 1.75 %; 669.13 us – 711.74 us
  - 15x15: 5.68 ms +- 0.49 %; 5.65 ms – 5.75 ms
  - 20x20: 28.64 ms +- 0.48 %; 28.44 ms – 28.82 ms
  - 30x30: oom
  - book: 178.51 ms +- 0.33 %; 177.46 ms - 179.62 ms
- fxhashmap
  - small: 39.12 us +- 10.26 %; 36.28 us – 49.70 us
  - wikipedia: 641.78 us +- 1.94 %; 630.47 us – 675.82 us
  - 15x15: 5.35 ms +- 0.80 %; 5.29 ms – 5.41 ms
  - 20x20: 27.49 ms +- 0.67 %; 27.20 ms – 27.81 ms
  - 30x30: oom
  - book: 142.74 ms +- 1.42 %; 140.26 ms – 146.90 ms
- better_vecs
  - small: 34.61 us +- 8.17 %; 31.95 us – 41.03 us
  - wikipedia: 597.71 us +- 2.06 %; 585.46 us – 631.27 us
  - 15x15: 5.75 ms +- 0.45 %; 5.73 ms – 5.80 ms
  - 20x20: 32.85 ms +- 0.31 %; 32.75 ms – 33.04 ms
  - 30x30: oom
  - book: 88.97 ms +- 1.18 %; 87.35 ms - 91.70 ms

## Todo

- re-do benchmarks
- investigate oom -> endless recursive loop?
- invent new solvers
