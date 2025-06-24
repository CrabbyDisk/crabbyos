	.global _entry
	.extern _stack_end

	.section .text.boot

_entry:	la sp, _stack_end
	jal _init
	j .
