# PineTime Tock

An out-of-tree port of Tock to the [PineTime](https://www.pine64.org/pinetime) smart watch.

## PineTime Resources

* [Wiki](https://wiki.pine64.org/index.php/PineTime)
* [Schematics](http://files.pine64.org/doc/PineTime/PineTime%20Schematic-V1.0a-20191103.pdf)
* [Pinout](http://files.pine64.org/doc/PineTime/PineTime%20Port%20Assignment%20rev1.0.pdf)

## Debugging (JLink)

1. Build kernel with debug symbols

    ```shell
    $ make -C board/pinetime debug
    ```

1. Start JLink GDB server in a seperate shell

    ```shell
    $ ./scripts/start_gdb_server_jlink.sh
    ```

1. Debug with `arm-none-eabi-gdb`

    ```shell
    $ ./scripts/gdb.sh
    # Alternatively, debug with `cgdb` TUI interface
    $ ./scripts/cgdb.sh
    ```
