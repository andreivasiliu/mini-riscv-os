.section .text

.global _start
.global usart_put_byte
.global usart_get_byte
.global flash_write
.global flash_page_erase

_start:
    call    disable_interrupts
    call    init_clocks
    call    init_blue_led
    call    init_red_led
    call    init_green_led
    call    turn_on_blue_led
    call    init_usart
    call    turn_off_blue_led
    call    turn_off_red_led
    call    turn_off_green_led
    call    init_globals
    call    setup_interrupts

    la      a0, helloworld
    li      a1, 40
    call    usart_send_string

    call    show_interrupt_info

    li      sp, 0x20000000 + 30 * 1024

    call    os_main
    call    cycle_blue_led

exit:
    wfi
    j       exit

# --------

disable_interrupts:
    csrwi   mie, 0          # Disable machine interrupt-pending
    csrwi   mip, 0          # Disable machine interrupt-enabled
    ret

# --------

setup_interrupts:
    la      a0, interrupt_handler
    csrw    mtvec, a0
    csrwi   mie, 1 << 3     # Enable software interrupts
    ret

interrupt_handler:
    # Save basic registers
    csrw    mscratch, sp
    li      sp, 0x20000000 + 31 * 1024
    sw      ra, -4(sp)
    sw      a0, -8(sp)
    sw      a1, -12(sp)
    sw      a2, -16(sp)
    sw      a3, -20(sp)
    sw      a4, -24(sp)
    sw      a5, -28(sp)
    sw      a6, -32(sp)
    sw      t0, -36(sp)
    sw      t1, -40(sp)

    # Skip program counter over the ecall instruction
    csrr    a0, mepc
    addi    a0, a0, 4
    csrw    mepc, a0

    csrr    a2, mcause
    addi    a2, a2, -11     # Environment call from M-mode
    beqz    a2, 1f

    # Not M-mode ecall; reject and return
    la      a0, bios_prefix
    li      a1, 16
    call    usart_send_string
    la      a0, unknown_interrupt_taken
    li      a1, 26
    call    usart_send_string
    j       ecall_end

1:  # Check system call number
    lw      a0, -8(sp)
    addi    a0, a0, -1
    beqz    a0, syscall_delay
    addi    a0, a0, -1
    beqz    a0, syscall_leds
    addi    a0, a0, -1
    beqz    a0, syscall_exec
    addi    a0, a0, -1
    beqz    a0, syscall_exit
    addi    a0, a0, -1
    beqz    a0, syscall_put_byte

ecall_end:
    # Restore registers
    lw      ra, -4(sp)
    lw      a0, -8(sp)
    lw      a1, -12(sp)
    lw      a2, -16(sp)
    lw      a3, -20(sp)
    lw      a4, -24(sp)
    lw      a5, -28(sp)
    lw      a6, -32(sp)
    lw      t0, -36(sp)
    lw      t1, -40(sp)
    csrr    sp, mscratch

    mret

# --------

# System calls

syscall_delay:
    # Print "Syscall: Delay"
    la      a0, bios_prefix
    li      a1, 16
    call    usart_send_string
    la      a0, ascii_syscall_delay
    li      a1, 16
    call    usart_send_string

    # Syscall arguments
    lw      a2, -12(sp)     # Arg 1: Milliseconds

    li      a0, 2000
    mul     a2, a2, a0

1:  addi    a2, a2, -1
    bnez    a2, 1b

    j       ecall_end


syscall_leds:
    # Print "Syscall: Set LEDs"
    la      a0, bios_prefix
    li      a1, 16
    call    usart_send_string
    la      a0, ascii_syscall_set_leds
    li      a1, 19
    call    usart_send_string

    # Syscall arguments
    lw      a2, -12(sp)     # Arg 1: Led mask

    # Turn on red led
    andi    a3, a2, 0b000001
    beqz    a3, 2f
    call    turn_on_red_led

2:  # Turn on green led
    andi    a3, a2, 0b000010
    beqz    a3, 3f
    call    turn_on_green_led

3:  # Turn on blue led
    andi    a3, a2, 0b000100
    beqz    a3, 4f
    call    turn_on_blue_led

4:   # Turn off red led
    andi    a3, a2, 0b001000
    beqz    a3, 5f
    call    turn_off_red_led

5:  # Turn off green led
    andi    a3, a2, 0b010000
    beqz    a3, 6f
    call    turn_off_green_led

6:  # Turn off blue led
    andi    a3, a2, 0b100000
    beqz    a3, 7f
    call    turn_off_blue_led

7:  j       ecall_end


syscall_exec:
    # Print "Syscall: Exec"
    la      a0, bios_prefix
    li      a1, 16
    call    usart_send_string
    la      a0, ascii_syscall_exec
    li      a1, 15
    call    usart_send_string

    li      a0, 0x20000000 + 31 * 1024  # Globals
    addi    a1, a0, 0x04    # Execution stack location

    lw      a2, 0x00(a0)    # Execution stack depth
    add     a3, a1, a2      # Current save point

    csrr    a4, mepc        # Current program counter
    sw      a4, (a3)        # Save to save point
    addi    a2, a2, 0x04    # Increase depth
    sw      a2, 0x00(a0)    # Save new stack depth

    # Syscall arguments
    lw      a5, -12(sp)     # Arg 1: Execution address

    csrw    mepc, a5        # Change return address

    j       ecall_end


syscall_exit:
    # Print "Syscall: Exit"
    la      a0, bios_prefix
    li      a1, 16
    call    usart_send_string
    la      a0, ascii_syscall_exit
    li      a1, 15
    call    usart_send_string

    li      a0, 0x20000000 + 31 * 1024  # Globals
    addi    a1, a0, 0x04    # Execution stack location

    lw      a2, 0x00(a0)    # Execution stack depth
    addi    a2, a2, -0x04   # Decrease depth
    sw      a2, 0x00(a0)    # Save new stack depth

    add     a3, a1, a2      # Last save point
    lw      a4, (a3)        # Load from save point

    # Syscall arguments
    lw      a5, -12(sp)     # Arg 1: Exit code (ignored)

    csrw    mepc, a4        # Change return address

    j       ecall_end


syscall_put_byte:
    # Syscall arguments
    lw      a6, -12(sp)     # Arg 1: Byte/character to print

    call    usart_put_byte

    j       ecall_end

# --------

# RCU base: 0x4002 1000

init_clocks:
    li      a0, 0x40021000  # RCU base (Reset and Clock Unit)
    lw      a1, 0x18(a0)    # Load RCU_APB2EN (advanced peripheral bus 2)
    li      a2, 1 << 14     # Enable USART0EN (USART0 clock enable)
    or      a1, a1, a2
    ori     a1, a1, 1 << 2  # Enable PAEN (GPIO port A clock enable)
    ori     a1, a1, 1 << 4  # Enable PCEN (GPIO port C clock enable)
    # ori     a1, a1, 1 << 0  # Enable AFEN (Alternate function IO clock enable)
    sw      a1, 0x18(a0)    # Store in RCU_APB2EN

    ret

# --------

init_globals:
    li      a0, 0x20000000 + 31 * 1024

    sw      x0, 0x00(a0)    # Execution stack depth

    ret

# --------

# GPIOA base: 0x4001 0800
# PC13: Red LED (GPIOC, CTL1, bit shift 20)
# PA1: Green LED (GPIOA, CTL0, bit shift 4)
# PA2: Blue LED (GPIOA, CTL0, bit shift 8)

init_red_led:
    li      a0, 0x40011000  # GPIOC_BASE

    lw      a1, 0x04(a0)    # Load GPIOC_CTL1
    li      a2, ~(0b1111 << 20) # Shift 20 for GPIO PA2 (red led)
    li      a3, 0b0011 << 20 # Configure as output, push-pull, 50MHz
    and     a1, a1, a2      # Clear previous configuration
    or      a1, a1, a3      # Set new configuration
    sw      a1, 0x04(a0)    # Store to GPIOC_CTL1

    ret

turn_on_red_led:
    li      a0, 0x40011000  # GPIOC_BASE

    li      a1, 1 << 13     # Select PC13 (red led)
    sw      a1, 0x14(a0)    # Set to GPIOx_BC (bit clear, turn on)

    ret

turn_off_red_led:
    li      a0, 0x40011000  # GPIOC_BASE

    li      a1, 1 << 13     # Select PC13 (red led)
    sw      a1, 0x10(a0)    # GPIOx_BOP (bit operate, turn off)

    ret


init_green_led:
    li      a0, 0x40010800  # GPIOA_BASE

    lw      a1, (a0)        # Load GPIOA_CTL0
    li      a2, ~(0b1111 << 4) # Shift 4 for GPIO PA2 (green led)
    li      a3, 0b0011 << 4 # Configure as output, push-pull, 50MHz
    and     a1, a1, a2      # Clear previous configuration
    or      a1, a1, a3      # Set new configuration
    sw      a1, (a0)        # Store to GPIOA_CTL0

    ret

turn_on_green_led:
    li      a0, 0x40010800  # GPIOA_BASE

    li      a1, 1 << 1      # Select PA1 (green led)
    sw      a1, 0x14(a0)    # Set to GPIOx_BC (bit clear, turn on)

    ret

turn_off_green_led:
    li      a0, 0x40010800  # GPIOA_BASE

    li      a1, 1 << 1      # Select PA1 (green led)
    sw      a1, 0x10(a0)    # GPIOx_BOP (bit operate, turn off)

    ret


init_blue_led:
    li      a0, 0x40010800  # GPIOA_BASE

    lw      a1, (a0)        # Load GPIOA_CTL0
    li      a2, ~(0b1111 << 8) # Shift 8 for GPIO PA2 (blue led)
    li      a3, 0b0011 << 8 # Configure as output, push-pull, 50MHz
    and     a1, a1, a2      # Clear previous configuration
    or      a1, a1, a3      # Set new configuration
    sw      a1, (a0)        # Store to GPIOA_CTL0

    ret

turn_on_blue_led:
    li      a0, 0x40010800  # GPIOA_BASE

    li      a1, 1 << 2      # Select PA2 (blue led)
    sw      a1, 0x14(a0)    # Set to GPIOx_BC (bit clear, turn on)

    ret

turn_off_blue_led:
    li      a0, 0x40010800  # GPIOA_BASE

    li      a1, 1 << 2      # Select PA2 (blue led)
    sw      a1, 0x10(a0)    # GPIOx_BOP (bit operate, turn off)

    ret

cycle_blue_led:
    call    turn_on_blue_led
    li      a0, 600
    call    delay
    call    turn_off_blue_led
    li      a0, 30000
    call    delay

    # la      a0, number_prompt
    # li      a1, 8
    # call    usart_send_string

    # csrr    a0, mcycle
    # call    send_number

    j       cycle_blue_led

delay:
    addi    a0, a0, -1
    bnez    a0, delay
    ret

# --------

# USART0 base: 0x4001 3800 - 0x4001 3BFF

# USART_STAT - offset: 0x00, reset: 0xC0
# USART_DATA - offset: 0x04, reset: undefined
# USART_BAUD - offset: 0x08, reset: 0x00
# USART_CTL0 - offset: 0x0C, reset: 0x00
# USART_CTL1 - offset: 0x10, reset: 0x00
# USART_CTL2 - offset: 0x14, reset: 0x00

init_usart:
    li      a2, 0x40010800  # GPIOA_BASE

    # Tx (PA9): Output, alternate function, push-pull, 50MHz
    # Rx (PA10): Input, floating
    lw      a3, 0x04(a2)    # GPIOA_CTL1
    li      a4, 0b1011 << 4 | 0b0100 << 8 
    li      a5, ~(0b1111 << 4 | 0b1111 << 8)
    and     a3, a3, a5
    or      a3, a3, a4
    sw      a3, 0x04(a2)    # Set to GPIO PA9/PA10 (tx, rx)

    # Write WL in USART_CTL0 to set data bits length
    # ..already defaults to 8..

    # Set the STB[1:0] bits in USART_CTL1 to set stop bits
    # ..already defaults to 1..

    # Set the baud rate in USART_BAUD to 108_000_000 / 115200 = 3a9.8
    li      a2, 0x40013800  # USART0 base
    li      a3, 69          # 8_000_000 / 115_200
    sw      a3, 0x08(a2)    # USART_BAUD

    # Set the TEN (transmission enable) bit in USART_CTL0
    # Set the REN (receiver enable) bit in USART_CTL0
    # Set the UEN (USART enable) bit in USART_CTL0
    lw      a3, 0x0C(a2)
    li      a4, 1 << 2 | 1 << 3 | 1 << 13
    or      a3, a3, a4
    sw      a3, 0x0C(a2)

    ret

usart_send_string:
    # Init string offset at 0
    li      a4, 0

1:
    # Finish when byte offset equals length
    beq     a4, a1, 2f

    # Load byte from string
    add     a5, a0, a4
    lb      a6, (a5)

    mv      s1, ra
    call    usart_put_byte
    mv      ra, s1

    # Increment byte offset
    addi    a4, a4, 1

    j       1b

2:  # End
    ret

usart_put_byte:
    # Wait for the TBE (transmit buffer empty) to be asserted
    li      t0, 0x40013800  # USART0 base
    lw      t1, 0x00(t0)    # USART_STAT
    andi    t1, t1, 1 << 7
    beqz    t1, usart_put_byte

    # Store byte in USART_DATA
    sb      a6, 0x04(t0)
    
    ret

usart_end:
    # Wait for the TC (transmission complete) to be asserted
    lw      a3, 0x00(a2)
    andi    a3, a3, 1 << 7
    beqz    a3, usart_end

    ret

usart_get_byte:
    # Wait for RBNE (read data buffer not empty) to be asserted
    li      t0, 0x40013800  # USART0 base
    lw      t1, 0x00(t0)    # USART_STAT
    andi    t1, t1, 1 << 5  # RBNE bit
    beqz    t1, usart_get_byte

    # Check status and errors
    lw      t1, 0x00(t0)    # USART_STAT
    andi    t1, t1, 0b111   # Parity, frame, or noise error
    beqz    t1, 1f
    lb      t1, 0x04(t0)    # Discard byte
    j       usart_get_byte

    # Load byte from USART_DATA
1:  lb      a6, 0x04(t0)

    ret

# --------

send_hello_world:
    la      a0, helloworld
    li      a1, 24
    j       usart_send_string

# --------

send_error:
    la      a0, error
    li      a1, 11
    call    usart_send_string

    j       4f

send_number:
    mv      s0, ra

    bgez    a0, 1f

    # Negative
    la      a6, '-'
    call    usart_put_byte

    neg     a0, a0

1:  # Positive
    li      a1, 10      # Divisor (base)
    li      a2, 0       # Reversed number
    li      a5, 0       # Digit count
    li      a7, 10      # Digit limit

2:  # Reverse digit
    mul     a2, a2, a1
    remu    a3, a0, a1
    add     a2, a2, a3
    divu    a0, a0, a1
    addi    a5, a5, 1
    bgt     a5, a7, send_error
    bnez    a0, 2b
    mv      a0, a2

3:  # Send digit
    rem     a6, a0, a1
    addi    a6, a6, '0'
    div     a0, a0, a1
    addi    a5, a5, -1

    call    usart_put_byte

    bgtz    a5, 3b

    la      a6, '\r'
    call    usart_put_byte
    la      a6, '\n'
    call    usart_put_byte
    # call    usart_end

4:  # End
    mv      ra, s0
    ret

# --------

# FMC base address: 0x4002 2000 (flash memory controller)
# 

flash_page_erase:
    # Input: a0 - page number

    # Unlock if necessary (if LK is set)
    la      a1, 0x40022000  # FMC base
    li      a2, 0x45670123  # Load unlock key 1
    sw      a2, 0x04(a1)    # Store into FMC_KEY
    li      a2, 0xCDEF89AB  # Load unlock key 2
    sw      a2, 0x04(a1)    # Store into FMC_KEY

    # Check BUSY bit in FMC_STAT (must be 0)
1:  lw      a2, 0x0C(a1)    # FMC_STAT
    andi    a2, a2, 1 << 0  # BUSY bit
    bnez    a2, 1b

    # Set the PER bit in FMC_CTL
    lw      a2, 0x10(a1)    # FMC_CTL
    ori     a2, a2, 1 << 1  # Set PER (main flash page erase)
    sw      a2, 0x10(a1)

    # Write page absolute address into the FMC_ADDR register
    li      a2, 0x08000000
    slli    a0, a0, 10      # Multiply by 1024 (block size)
    add     a2, a2, a0      # Get block address
    sw      a2, 0x14(a1)    # Store into FMC_ADDR

    # Send the page erase command by setting START in FMC_CTL
    lw      a2, 0x10(a1)    # FMC_CTL
    ori     a2, a2, 1 << 6  # Set START bit
    sw      a2, 0x10(a1)

    # Check the BUSY bit
1:  lw      a2, 0x0C(a1)    # FMC_STAT
    andi    a2, a2, 1 << 0  # BUSY bit
    bnez    a2, 1b

    # Lock
    lw      a2, 0x10(a1)    # FMC_CTL
    li      a3, ~(1 << 0)   # Clear PER bit
    and     a2, a2, a3
    ori     a2, a2, 1 << 7  # Set LK bit (lock)
    sw      a2, 0x10(a1)

    ret

flash_write:
    # Input: a0 - source page number, a1 - target page number

    # Unlock if necessary (if LK is set)
    la      a2, 0x40022000  # FMC base
    li     a3, 0x45670123  # Load unlock key 1
    sw      a3, 0x04(a2)    # Store into FMC_KEY
    li     a3, 0xCDEF89AB  # Load unlock key 2
    sw      a3, 0x04(a2)    # Store into FMC_KEY

    # Check BUSY bit in FMC_STAT (must be 0)
1:  lw      a3, 0x0C(a2)    # FMC_STAT
    andi    a3, a3, 1 << 0  # BUSY bit
    bnez    a3, 1b

    # Get destination page address
    li      a6, 0x08000000
    slli    a1, a1, 10      # Multiply by 1024 (block size)
    add     a6, a6, a1      # Get block address

    # Get source memory address
    li      a4, 0x20000000
    slli    a0, a0, 10
    add     a4, a4, a0

    # Set limit
    addi    a0, a4, 1024

1:  # Set the PG bit in FMC_CTL
    lw      a3, 0x10(a2)    # FMC_CTL
    ori     a3, a3, 1 << 0  # Set PG (main flash program)
    sw      a3, 0x10(a2)

    # Write 16-bit/32-bit value at 0x08XXXXXX
    lw      a5, 0x00(a4)    # Load data from RAM
    sw      a5, 0x00(a6)    # Store data into flash

    # Check BUSY
2:  lw      a3, 0x0C(a2)    # FMC_STAT
    andi    a3, a3, 1 << 0  # BUSY bit
    bnez    a3, 2b

    # Increment counter and loop 1024 times
    addi    a6, a6, 4
    addi    a4, a4, 4
    bne     a4, a0, 1b

    # Lock
    lw      a3, 0x10(a2)    # FMC_CTL
    li      a4, ~(1 << 0)   # Clear PG bit
    and     a3, a3, a4
    ori     a3, a3, 1 << 7  # Set LK bit (lock)
    sw      a3, 0x10(a2)

    ret

# --------

.macro print message, length, csr
    la      a0, bios_prefix
    li      a1, 16
    call    usart_send_string

    la      a0, \message
    li      a1, \length
    call    usart_send_string

    csrr    a0, \csr
    call    send_number
.endm

show_interrupt_info:
    mv      s2, ra

    print   cycle_count, 13, mcycle
    print   trap_cause, 12, mcause
    print   interrupt_vector, 18, mtvec
    print   return_address, 16, mepc

    mv      ra, s2
    ret

# --------

helloworld:
    .ascii  "\r\n[\33[35mbios\33[0m] \33[35mHello world\33[0m\r\n"

number_prompt:
    .ascii  "Number: "

bios_prefix:
    .ascii  "[\33[35mbios\33[0m] "

cycle_count:
    .ascii  "Cycle count: "

trap_cause:
    .ascii  "Trap cause: "

interrupt_vector:
    .ascii  "Interrupt vector: "

return_address:
    .ascii  "Return address: "

interrupt_taken:
    .ascii  "Interrupt taken!\r\n"

ascii_syscall_set_leds:
    .ascii  "Syscall: Set LEDs\r\n"

ascii_syscall_delay:
    .ascii  "Syscall: Delay\r\n"

ascii_syscall_exec:
    .ascii  "Syscall: Exec\r\n"

ascii_syscall_exit:
    .ascii  "Syscall: Exit\r\n"

unknown_interrupt_taken:
    .ascii  "Unknown interrupt taken!\r\n"

error:
    .ascii  "<too big>\r\n"

negative:
    .ascii  "<negative>\r\n"

