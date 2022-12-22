Small example using Slint on  the MXCHIP IoT DevKit (AZ3166)


To Build and run:

You need:
 - Rust nightly (because `#![feature(default_alloc_error_handler)]`) with the thumbv7em-none-eabihf toolchain
 - openocd
 - GDB for arm

In a terminal tab:

```
openocd -f interface/stlink-v2-1.cfg -f target/stm32f4x.cfg
```

In another terminal

```
cargo +nightly  build --release
gdb-multiarch -q target/thumbv7em-none-eabihf/release/carousel  -ex "target remote localhost:3333"  -ex "load" -ex "continue"
```

