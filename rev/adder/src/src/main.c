# include "program.h"
# include <stdlib.h>
# include <stdio.h>

# define TAPE_LENGTH 1024

# define STATE_LENGTH 26
# define SYMBOL_LENGTH 5
# define MOVE_LENGTH 2

# define MASK(length) ((unsigned long) (1 << length) - 1)
# define RADIX(from, read) ((from << SYMBOL_LENGTH) | read)

# define I_FROM(i) (((MASK(STATE_LENGTH) << 38) & i) >> 38)
# define I_READ(i) (((MASK(SYMBOL_LENGTH) << 33) & i) >> 33)
# define I_WRITE(i) (((MASK(SYMBOL_LENGTH) << 28) & i) >> 28)
# define I_MOVE(i) (((MASK(MOVE_LENGTH) << 26) & i) >> 26)
# define I_TO(i) (((MASK(STATE_LENGTH) << 0) & i) >> 0)

int lookup(unsigned int from, unsigned read) {
    unsigned int search = RADIX(from, read);

    int left = 0;
    int right = PROGRAM_LENGTH;
    while (left < right) {
        int middle = (left + right) / 2;

        unsigned long instruction = program[middle];

        unsigned int value = RADIX(
            I_FROM(instruction),
            I_READ(instruction)
        );

        if (search == value) return middle;
        else if (search < value) right = middle;
        else left = middle + 1;
    }
    return -1;
}

int check_license() {
    char buffer[256];
    printf("license: ");
    fflush(stdout);
    fgets(buffer, 256, stdin);

    unsigned short* tape = calloc(TAPE_LENGTH, sizeof(unsigned short));

    tape[0] = BEGIN_SYMBOL;

    for (int i = 0; i < 256; i++) {
        char c = buffer[i];

        if (c == '\0' || c == '\n') {
            if (i == 0) {
                puts("license is required!");
                return 0;
            }

            tape[i * 2 + 1] = END_SYMBOL;
            break;
        }

        tape[i * 2 + 1] = (c & 0xf0) >> 4;
        tape[i * 2 + 2] = c & 0x0f;
    }

    int state = 0;
    int head = 0;

    while (state != ACCEPT_STATE && state != REJECT_STATE) {
        if (head < 0 || head >= TAPE_LENGTH) {
            puts("error while checking license!");
            return 0;
        }

        int next_index = lookup(state, tape[head]);

        if (next_index == -1) {
            puts("error while checking license!");
            return 0;
        }

        unsigned long instruction = program[next_index];

        tape[head] = I_WRITE(instruction);
        if (I_MOVE(instruction) == 2) head -= 1;
        if (I_MOVE(instruction) == 1) head += 1;
        state = I_TO(instruction);
    }

    if (state != ACCEPT_STATE) {
        puts("license is invalid.");
        return 0;
    }

    puts("license is valid!");
    return 1;
}

int main() {
    puts("welcome to the proprietary adder!");

    if (!check_license()) return 0;

    int a = 0;
    int b = 0;
    printf("first number: ");
    fflush(stdout);
    scanf("%d", &a);
    printf("second number: ");
    fflush(stdout);
    scanf("%d", &b);
    printf("result: %d\n", a + b);

    return 0;
}
