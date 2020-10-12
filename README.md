korg-syro-rs
============

Rust API for the
[KORG SYRO](https://github.com/korginc/volcasample)
library for the Volca Sample.

## TODO

* Limit number of operations
    > To send multiple data, create an array of SyroData structures and set the above information for each one. A maximum of 110 SyroData structures can be transferred in one operation.

* Memory usage estimation
    > Memory Size for Sample   4 MB, Maximum 65 seconds