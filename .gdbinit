tui enable
define reload
	target extended-remote localhost:1234
	directory
	add-symbol-file target/riscv64gc-unknown-none-elf/release/crabbyos
	continue
end
set pagination off
add-symbol-file target/riscv64gc-unknown-none-elf/release/crabbyos
br _init
target extended-remote localhost:1234
continue
