# limbo_alloc


```
std_alloc_1000          time:   [79.989 µs 81.518 µs 83.117 µs]
                        change: [-8.5289% -4.7598% -1.0164%] (p = 0.02 < 0.05)
                        Performance has improved.
Found 5 outliers among 100 measurements (5.00%)
  4 (4.00%) high mild
  1 (1.00%) high severe

limbo_alloc_1000        time:   [23.918 µs 24.262 µs 24.605 µs]
                        change: [-5.5812% -2.8784% -0.1139%] (p = 0.05 < 0.05)
                        Change within noise threshold.
Found 6 outliers among 100 measurements (6.00%)
  5 (5.00%) high mild
  1 (1.00%) high severe

std_alloc_10000         time:   [2.6323 ms 2.6751 ms 2.7190 ms]
                        change: [-5.8254% -1.5052% +3.0683%] (p = 0.52 > 0.05)
                        No change in performance detected.
Found 6 outliers among 100 measurements (6.00%)
  2 (2.00%) high mild
  4 (4.00%) high severe

limbo_alloc_10000       time:   [280.74 µs 288.79 µs 299.02 µs]
                        change: [-72.045% -70.139% -68.042%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 4 outliers among 100 measurements (4.00%)
  1 (1.00%) high mild
  3 (3.00%) high severe

std_alloc_100000        time:   [27.699 ms 28.118 ms 28.553 ms]
                        change: [-45.422% -35.382% -26.098%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild

Benchmarking limbo_alloc_100000: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 20.0s. You may wish to increase target time to 28.7s, enable flat sampling, or reduce sample count to 50.
limbo_alloc_100000      time:   [5.2828 ms 5.3769 ms 5.4842 ms]
                        change: [-20.977% -17.713% -14.054%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 8 outliers among 100 measurements (8.00%)
  7 (7.00%) high mild
  1 (1.00%) high severe

Benchmarking std_alloc_1000000: Warming up for 3.0000 s
Warning: Unable to complete 100 samples in 20.0s. You may wish to increase target time to 27.0s, or reduce sample count to 70.
std_alloc_1000000       time:   [266.68 ms 272.77 ms 278.83 ms]
                        change: [-10.874% -7.8000% -4.5806%] (p = 0.00 < 0.05)
                        Performance has improved.

limbo_alloc_1000000     time:   [41.092 ms 41.906 ms 42.776 ms]
                        change: [-17.052% -13.801% -10.526%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 5 outliers among 100 measurements (5.00%)
  5 (5.00%) high mild

```