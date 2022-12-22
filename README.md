Small example using [Slint](https://slint-ui.com) on  the MXCHIP IoT DevKit (AZ3166)


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

https://user-images.githubusercontent.com/959326/209118829-2bddee36-00c3-4c3d-90ab-d745a7f3504c.mp4
