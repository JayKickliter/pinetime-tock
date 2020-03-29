# The following code breaks on hook we have added to `main.rs` and
# observes process loading. It reads the process's name & `.text`
# section address and uses that information to:
#
# 1. Infer what `.elf` file we need to load to get debug symbols for the process
# 2. Load the symbols with correct `.text` offsets so GDB knows the
#    actual location of functions and data
#
# NOTE: This code does not load anything onto the chip. The processes
# are already on flash and Tock takes care of starting them.
break load_process_hook
commands
  set $APP_NAME = name.data_ptr
  # The `+ 40` below is to compensate for the fact that the Tock
  # kernel does not return the actual location of process `.text`
  # section, but the beginning of it's `APP_MEMORY` region.  To fix
  # this properly, we will need to add a method to Tock's `Processes`
  # type to return the correct address.
  set $APP_TEXT = text_addr + 40
  eval "add-symbol-file apps/%s/build/cortex-m4/cortex-m4.elf %d", $APP_NAME, $APP_TEXT
  tbreak main
  continue
end
