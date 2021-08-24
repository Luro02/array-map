array-map
=========

[![ci](https://github.com/Luro02/array-map/actions/workflows/ci.yml/badge.svg)](https://github.com/Luro02/array-map/actions/workflows/ci.yml)
[![Code Coverage](https://codecov.io/gh/Luro02/array-map/branch/master/graph/badge.svg)](https://codecov.io/gh/Luro02/array-map/branch/master)

[Latest Documentation](https://array-map.xdg.io/)

This crate provides an array based hashmap (`ArrayMap`), which works in a no-std environment and can be used if an upper bound
for the number of keys in the map is known at compile-time.

The crate requires the nightly compiler to be used, which is why it has not been published on [crates.io](https://crates.io/) yet.
See [#2](https://github.com/Luro02/array-map/issues/2).

### Credits

This project has been heavily inspired by https://github.com/rust-lang/hashbrown and some of the initial
documentation and tests have been copied from this project.
