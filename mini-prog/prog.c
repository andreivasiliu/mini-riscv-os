#include "syscalls.h"

void _start() {
    for(int i = 0; i < 15; i++) {
        sys_set_leds(RED | BLUE);
        sys_delay(100);
        sys_set_leds(GREEN);
        sys_delay(100);
    }

    sys_set_leds(0);
    sys_exit(0);
}
