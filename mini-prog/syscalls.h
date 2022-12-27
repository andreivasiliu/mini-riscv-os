static int ecall(int name, int arg1) {
    // register int res asm("a0");
    register long syscall_number asm("a0") = name;
    register int a1 asm("a1") = arg1;

    asm volatile (
        "ecall # Using %0 and %1"
        : "=r" (syscall_number)
        : "r" (syscall_number), "r" (a1)
    );

    return syscall_number;
}

#define RED 1
#define GREEN 2
#define BLUE 4


static int sys_set_leds(int leds) {
    ecall(2, 0b111000);
    ecall(2, leds);
}

static int sys_delay(int milliseconds) {
    ecall(1, milliseconds);
}

static int sys_exit(int exit_code) {
    ecall(4, exit_code);
}
