# arm64-tee

Example Rust application running in arm64 secure world EL1 and EL0

## Compile

Some unstable features are used, so to compile you must use [nightly rust
compiler](https://doc.rust-lang.org/1.2.0/book/nightly-rust.html).

In addition, a C cross compiler must be used. Configuration scripts assume the
prefix `aarch64-linux-gnu-`.

Image can then be compiled with:

~~~
$ make
~~~

Several `bl32.*` files are created. `bl32.bin` should be loaded by [Trusted
Firmware-A](https://github.com/ARM-software/arm-trusted-firmware.git). The
firmware can be compiled with:

~~~
$ make CROSS_COMPILE=aarch64-linux-gnu- PLAT=qemu SPD=opteed DEBUG=1 ARM_LINUX_KERNEL_AS_BL33=1 BL32=/path/to/bl32.img BL33=/path/to/some/kernel all fip
$ dd if=build/qemu/debug/bl1.bin of=flash.bin bs=4096 conv=notrunc
$ dd if=build/qemu/debug/fip.bin of=flash.bin seek=64 bs=4096 conv=notrunc
~~~

Qemu can be used to run the code:

~~~
$ qemu-system-aarch64 -nographic -machine virt,secure=on -cpu max -smp 1 -m 1024 -semihosting-config enable,target=native -no-acpi -bios flash.bin
~~~

## Minimal OS

[minimalos/](minimalos/) can be used as a test OS to check if the SMC interfaces
are working.

# Features

- Serial interface to print data
- OP-TEE interface to receive control from normal world
- Drop to S-EL0 on SMC
- Virtual memory set up via MMU
  - EL1-0 share `.text`
  - EL1-0 share `.rodata`
- Dynamic memory set up for EL0
  - Configured as global default allocator so that `collections` can be used
- SVC interface from EL0 to EL1
