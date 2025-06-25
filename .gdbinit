tui enable
define reload
	shell cargo build --release > /dev/null 2>&1
	directory
	symbol-file target/riscv64gc-unknown-none-elf/release/crabbyos
	target extended-remote localhost:1234
	continue
end
set pagination off
add-symbol-file target/riscv64gc-unknown-none-elf/release/crabbyos
br _init
target extended-remote localhost:1234
continue
