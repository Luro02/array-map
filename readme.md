array-map
=========

[![ci](https://github.com/Luro02/array-map/actions/workflows/ci.yml/badge.svg)](https://github.com/Luro02/array-map/actions/workflows/ci.yml)
[![Code Coverage](https://codecov.io/gh/Luro02/array-map/branch/master/graph/badge.svg)](https://codecov.io/gh/Luro02/array-map/branch/master)

This crate provides an array based hashmap (`ArrayMap`), which works in a no-std environment and can be used if an upper bound
for the number of keys in the map is known at compile-time.


### Credits

This project has been heavily inspired by https://github.com/rust-lang/hashbrown and some of the initial
documentation and tests have been copied from this project.
