https://github.com/cross-rs/cross

Tested Android Version 9(aarch64) APi Level 28 Linux Kernel 4.4.111-21340847

1. cross build --target=arm-linux-androideabi --release
2. cd target/arm-linux-androideabi/release
3. adb push client /data/local/tmp
4. adb shell
5. (adb) su
6. (adb) cd /data/local/tmp
7. (adb) ./client

