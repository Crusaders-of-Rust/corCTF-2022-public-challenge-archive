import random

def matrix(a):
    return [
        (a & (1 << i)) >> i
        for i in range(4)
    ]

def plus(a, b):
    return a ^ b

def times(a, b):
    m1 = matrix(a)
    m2 = matrix(b)
    return sum(
        (1 << i) * v
        for i, v in enumerate([
            (m1[0] & m2[0]) ^ (m1[1] & m2[2]),
            (m1[0] & m2[1]) ^ (m1[1] & m2[3]),
            (m1[2] & m2[0]) ^ (m1[3] & m2[2]),
            (m1[2] & m2[1]) ^ (m1[3] & m2[3]),
        ])
    )


# special tape symbols
## characters are 0-15
## blank space
BLANK = 16
## beginning of input
BEGIN_S = 30
## end of input
END_S = 31
## accept state
ACCEPT_S = 0b11111111111111111111111111
REJECT_S = 0b11111111111111111111111110
## default marker
MARKER = 20

# gadgets (return [transition, start, end])
## default state
DEFAULT_S = 'default'
## movement
LEFT = -1
NONE = 0
RIGHT = 1

def left():
    return {
        0: {
            DEFAULT_S: (1, None, LEFT),
        }
    }

def right():
    return {
        0: {
            DEFAULT_S: (1, None, RIGHT),
        }
    }

def left_until(symbol, invert=False):
    return {
        0: {
            symbol: (1 if invert else 2, None, NONE),
            DEFAULT_S: (2 if invert else 1, None, NONE),
        },
        1: {
            DEFAULT_S: (0, None, LEFT),
        }
    }

def right_until(symbol, invert=False):
    return {
        0: {
            symbol: (1 if invert else 2, None, NONE),
            DEFAULT_S: (2 if invert else 1, None, NONE),
        },
        1: {
            DEFAULT_S: (0, None, RIGHT),
        }
    }

def set(symbol):
    return {
        0: {
            DEFAULT_S: (1, symbol, NONE),
        }
    }

def inc(value):
    return {
        0: {
            i: (1, (i + value) % 16, NONE)
            for i in range(0, 16)
        }
    }

def dec(value):
    return {
        0: {
            i: (1, (i - value) % 16, NONE)
            for i in range(0, 16)
        }
    }

def add():
    return {
        0: {
            i: (i + 1, None, RIGHT)
            for i in range(0, 16)
        }
    } | {
        i + 1: {
            j: (17, plus(i, j), NONE)
            for j in range(0, 16)
        }
        for i in range(0, 16)
    }

def add_offset():
    return {
        0: {
            i: (i + 1, None, RIGHT)
            for i in range(0, 16)
        }
    } | {
        i + 1: {
            j: (17 + i, None, RIGHT)
            for j in range(0, 16)
        }
        for i in range(0, 16)
    } | {
        i + 17: {
            j: (16 + 17, plus(i, j), NONE)
            for j in range(0, 16)
        }
        for i in range(0, 16)
    }


def mult():
    return {
        0: {
            i: (i + 1, None, RIGHT)
            for i in range(0, 16)
        }
    } | {
        i + 1: {
            j: (17, times(i, j), NONE)
            for j in range(0, 16)
        }
        for i in range(0, 16)
    }


def accept():
    return {
        0: {
            DEFAULT_S: (ACCEPT_S, None, NONE),
        }
    }


def reject():
    return {
        0: {
            DEFAULT_S: (REJECT_S, None, NONE),
        }
    }


# shift gadget states by base address
def shift(gadget, base):
    new = {}
    total = 0
    special = [
        REJECT_S,
        ACCEPT_S,
    ]
    for start, transition in gadget.items():
        start_abs = start + base
        if start in special:
            start_abs = start
        for (symbol, target) in transition.items():
            (end, write, move) = target[:3]

            end_abs = end + base
            if end in special:
                end_abs = end
            if start_abs not in new:
                new[start_abs] = {}
            if symbol == DEFAULT_S:
                for i in range(32):
                    if i in new[start_abs]:
                        continue

                    if len(target) == 4:
                        new[start_abs][i] = (end_abs, write, move, target[3])
                    else:
                        new[start_abs][i] = (end_abs, write, move)
            else:
                if len(target) == 4:
                    new[start_abs][symbol] = (end_abs, write, move, target[3])
                else:
                    new[start_abs][symbol] = (end_abs, write, move)
        total += 1
    return (new, total)


# link gadgets together
def link(*gadgets):
    function = {}

    # count up states for uniqueness
    states = 0

    for gadget in gadgets:
        offset = states
        new, total = shift(gadget, offset)
        function |= new
        states += total

    return function

# utility
def nop():
    return {
        0: {
            DEFAULT_S: (1, None, NONE),
        }
    }

def debug(out):
    return {
        0: {
            DEFAULT_S: (1, None, NONE, out),
        }
    }

def local(offset, look_left=False):
    return link(
        (left_until if look_left else right_until)(END_S),
        *[right() for _ in range(offset + 1)]
    )

def load(array):
    return link(*(
        link(
            set(i),
            right(),
        )
        for i in array
    ))

def beginning():
    return link(
        left_until(BEGIN_S),
        right(),
    )

def save_location(marker, r=False):
    def inner(*gadgets):
        linked = link(*gadgets or [nop()])

        # each sub-transition-function has reserved states
        end = max(linked.keys()) + 1
        width = end + 1

        # starts are 1, width + 1, 2 * width + 1, etc
        transition = {
            0: {
                i: (i * width + 1, marker, NONE)
                for i in range(32)
            }
        }

        for i in range(32):
            shifted, _ = shift(linked, i * width + 1)
            transition |= shifted

        # ends are at end + 1, width + end + 1, 2 * width + end + 1, etc
        transition |= {
            i * width + end + 1: {
                marker: (31 * width + end + 2, i, NONE),
                DEFAULT_S: (i * width + end + 1, None, RIGHT if r else LEFT),
            }
            for i in range(32)
        }
        return transition

    return inner

def hold():
    def inner(*gadgets):
        linked = link(*gadgets or [nop()])

        # each sub-transition-function has reserved states
        end = max(linked.keys()) + 1
        width = end + 1

        # starts are 1, width + 1, 2 * width + 1, etc
        transition = {
            0: {
                i: (i * width + 1, None, NONE)
                for i in range(32)
            }
        }

        for i in range(32):
            shifted, _ = shift(linked, i * width + 1)
            transition |= shifted

        # ends are at end + 1, width + end + 1, 2 * width + end + 1, etc
        transition |= {
            i * width + end + 1: {
                DEFAULT_S: (31 * width + end + 2, i, NONE),
            }
            for i in range(32)
        }
        return transition
    return inner


def do_at(offset, marker, r=False):
    def inner(*gadgets):
        return save_location(marker, r=r)(
            local(offset, look_left=r),
            *gadgets,
        )
    return inner


# branching
def local_compare(end_offset, marker, target=0, invert=False):
    def inner(*gadgets):
        read_and_return = {
            0: {
                i: (i + 1, marker, NONE)
                for i in range(32)
            },
        } | {
            i + 1: {
                END_S: ((i + 1) * (end_offset + 1) + 1 + 32 - 1, None, RIGHT),
                DEFAULT_S: (i + 1, None, RIGHT),
            }
            for i in range(32)
        } | {
            i * (end_offset + 1) + j + 1 + 32: {
                DEFAULT_S: (i * (end_offset + 1) + j + 1 + 32 - 1, None, RIGHT)
            }
            for i in range(32)
            for j in range(1, end_offset + 1)
        } | {
            i * (end_offset + 1) + 1 + 32: {
                target: (i * 2 + 32 * end_offset + 1 + 64, None, NONE),
                DEFAULT_S: (i * 2 + 1 + 32 * end_offset + 1 + 64, None, NONE),
            }
            for i in range(32)
        } | {
            i + 32 * end_offset + 1 + 64: {
                marker: (
                    64 + (32 * end_offset) + 1 + 64
                    if (i % 2 == 0) ^ invert else
                    64 + (32 * end_offset) + 1 + 64 + 1
                , i // 2, NONE),
                DEFAULT_S: (i + (32 * end_offset) + 1 + 64, None, LEFT),
            }
            for i in range(32 * 2)
        }

        # 64 + (32 * end_offset) + 1 + 64 + 1 means continue
        # 64 + (32 * end_offset) + 1 + 64 means done

        inside = link(*gadgets or [nop()])
        shifted, _ = shift(inside, 64 + (32 * end_offset) + 1 + 64 + 1)

        union = read_and_return | shifted
        union |= {
            64 + (32 * end_offset) + 1 + 64: {
                DEFAULT_S: (max(union.keys()) + 1, None, NONE)
            }
        }

        return union
    return inner

def local_loop(end_offset, marker, invert=False):
    def inner(*gadgets):
        read_and_return = {
            0: {
                i: (i + 1, marker, NONE)
                for i in range(32)
            },
        } | {
            i + 1: {
                END_S: ((i + 1) * (end_offset + 1) + 1 + 32 - 1, None, RIGHT),
                DEFAULT_S: (i + 1, None, RIGHT),
            }
            for i in range(32)
        } | {
            i * (end_offset + 1) + j + 1 + 32: {
                DEFAULT_S: (i * (end_offset + 1) + j + 1 + 32 - 1, None, RIGHT)
            }
            for i in range(32)
            for j in range(1, end_offset + 1)
        } | {
            i * (end_offset + 1) + 1 + 32: {
                0: (i * 2 + 32 * end_offset + 1 + 64, None, NONE),
                DEFAULT_S: (i * 2 + 1 + 32 * end_offset + 1 + 64, None, NONE),
            }
            for i in range(32)
        } | {
            i + 32 * end_offset + 1 + 64: {
                marker: (
                    64 + (32 * end_offset) + 1 + 64
                    if (i % 2 == 0) ^ invert else
                    64 + (32 * end_offset) + 1 + 64 + 1
                , i // 2, NONE),
                DEFAULT_S: (i + (32 * end_offset) + 1 + 64, None, LEFT),
            }
            for i in range(32 * 2)
        }

        # 64 + (32 * end_offset) + 1 + 64 + 1 means continue
        # 64 + (32 * end_offset) + 1 + 64 means done

        inside = link(*gadgets or [nop()])
        shifted, _ = shift(inside, 64 + (32 * end_offset) + 1 + 64 + 1)

        union = read_and_return | shifted

        final = max(union.keys()) + 1

        union |= {
            64 + (32 * end_offset) + 1 + 64: {
                DEFAULT_S: (final + 1, None, NONE)
            }
        }

        union |= {
            final: {
                DEFAULT_S: (0, None, NONE)
            }
        }

        return union
    return inner

def loop(value, invert=False):
    def inner(*gadgets):
        transition = {
            0: {
                value: (2 if invert else 1, None, NONE),
                DEFAULT_S: (1 if invert else 2, None, NONE),
            }
        }

        shifted, _ = shift(link(*gadgets or [nop()]), 2)
        transition |= shifted

        end = max(transition.keys()) + 1

        transition |= {
            1: {
                DEFAULT_S: (end + 1, None, NONE),
            },
            end: {
                DEFAULT_S: (0, None, NONE),
            },
        }

        return transition
    return inner

def compare(value):
    def outer(*left):
        def inner(*right):
            transition = {
                0: {
                    value: (2, None, NONE),
                    DEFAULT_S: (1, None, NONE),
                }
            }

            shifted, _ = shift(link(*left or [nop()]), 2)
            transition |= shifted

            end = max(transition.keys()) + 1

            shifted, _ = shift(link(*right or [nop()]), end + 1)
            transition |= shifted

            transition |= {
                1: {
                    DEFAULT_S: (end + 1, None, NONE)
                },
                end: {
                    DEFAULT_S: (max(transition.keys()) + 1, None, NONE)
                }
            }
            return transition
        return inner
    return outer


# run turing machine
def execute(tape, machine, debug=False):
    transition, start, (accept, reject) = machine

    state = start
    head = 0
    while state not in [accept, reject]:
        if debug:
            print(state, ' '.join([
                str(c) if i != head else f'[{c}]'
                for i, c in enumerate(tape)
            ]))

        symbol = tape[head]
        next, write, move = transition[state][symbol][:3]

        if len(transition[state][symbol]) == 4:
            print(transition[state][symbol][3])

        # go to next state
        state = next

        # write
        if write is not None:
            tape[head] = write

        # movement
        if move == LEFT:
            head -= 1
        if move == RIGHT:
            head += 1
    return state == accept

tape = (
    [BEGIN_S] +
    [
        int(i, 16) for i in
        '636f726374667b6131773479355f7337617433355f346e645f3761703335217d' ] +
    [END_S] +
    [0] * 256
)

target = [
    13, 11, 13, 15, 12, 13, 13, 10,
    6, 3, 6, 6, 13, 2, 2, 10,
    12, 9, 5, 13, 8, 11, 2, 3,
    12, 0, 11, 5, 5, 3, 12, 9,
    9, 15, 8, 10, 11, 12, 10, 14,
    8, 10, 6, 14, 1, 13, 15, 10,
    11, 0, 4, 4, 11, 10, 9, 9,
    12, 13, 12, 13, 4, 6, 5, 15,
]

transition = link(
    # set the flag to true
    local(0),
    set(1),
    left_until(BEGIN_S),

    # mark the end of counter buffer
    local(3),
    set(16),
    left_until(BEGIN_S),

    # count the number of characters
    # keep track of it in base 16
    local_loop(0, MARKER)(
        right(),
        compare(END_S)(
            # break
            local(0),
            set(0),
            beginning(),
        )(
            save_location(MARKER)(
                # go to counter
                local(1),
                # find proper digit
                loop(15, invert=True)(
                    set(0),
                    right(),
                ),

                # destroy end marker if hit
                compare(16)(
                    set(0)
                )(),

                # increment counter
                inc(1),
            ),
        ),
    ),

    # check that the length is correct
    local(1),
    compare(0)(
        set(0)
    )(
        save_location(MARKER, r=True)(
            local(0, look_left=True),
            set(1),
        ),
    ),
    right(),
    compare(4)(
        set(0)
    )(
        save_location(MARKER, r=True)(
            local(0, look_left=True),
            set(1),
        ),
    ),
    right(),
    compare(16)(
        set(0)
    )(
        save_location(MARKER, r=True)(
            local(0, look_left=True),
            set(1),
        ),
    ),

    local(0, look_left=True),
    compare(0)(
        # now, we know the length is correct
        beginning(),
        save_location(MARKER)(
            # put the matrix from locals 0-63
            local(0),
            load([
                6, 10, 1, 7, 7, 9, 2, 10,
                12, 7, 14, 7, 4, 14, 2,
                2, 5, 12, 2, 11, 2, 5, 15,
                7, 1, 10, 5, 9, 2, 14, 6,
                8, 13, 4, 5, 1, 12, 13, 9,
                5, 8, 4, 0, 9, 6, 14, 3,
                10, 7, 13, 8, 8, 9, 7, 11,
                4, 4, 10, 0, 11, 8, 4, 4, 2
            ]),

            # result matrix BACKWARDS in 64-127
            load([BLANK] * 64),
        ),

        # scratch space in 128 onward
        # 128: loop counter 1
        # 129: loop counter 2
        # 130: loop counter 3

        # 131-146: addition buffer

        # reset outer loop
        do_at(128, MARKER)(set(8)),
        local_loop(128, MARKER)(
            # block out the seen vector
            save_location(MARKER)(
                local(130),
                set(0),

                # clear the addition buffer
                right(),
                load([BLANK] * 18),

                loop(8)(
                    local(0, look_left=True),
                    right_until(BLANK, invert=True),

                    hold()(
                        set(BLANK),
                        local(131, look_left=True),
                        loop(16, invert=False)(
                            right(),
                            right(),
                        ),
                    ),

                    local(130, look_left=True),
                    inc(1),
                ),
            ),

            # cursor starts at beginning
            beginning(),

            # reset inner loop
            do_at(129, MARKER)(set(8)),
            local_loop(129, MARKER)(
                left(),
                save_location(MARKER)(
                    right(),

                    do_at(130, MARKER)(set(7)),
                    local_loop(130, MARKER)(

                        # copy to the proper place
                        *(copy := (
                            hold()(
                                set(MARKER),
                                local(132),
                                loop(16, invert=False)(
                                    right(),
                                    right(),
                                ),
                            ),
                            hold()(left_until(MARKER)),
                        )),

                        # decrement counter
                        do_at(130, MARKER)(dec(1)),

                        # move cursor to next
                        *(right() for _ in range(8)),
                    ),

                    # one more time
                    *copy
                ),
                right(),

                # do math
                # debug('multiplying...'),
                do_at(132, MARKER)(
                    loop(BLANK)(
                        left(),
                        mult(),

                        right(),
                        right(),
                    ),
                ),
                # debug('adding...'),
                do_at(134, MARKER)(
                    loop(BLANK)(
                        left(),
                        left(),
                        add_offset(),
                        right(),
                        right()
                    ),
                ),
                # debug('done adding!'),

                # push onto totals
                do_at(146, MARKER)(
                    hold()(
                        local(127, look_left=True),
                        left_until(BLANK),
                    ),
                ),

                # clear the sums
                do_at(132, MARKER)(
                    loop(BLANK)(
                        set(BLANK),
                        right(),
                        right(),
                    ),
                ),

                # next time, shift by one
                right(),

                # decrement counter
                do_at(129, MARKER)(dec(1)),
            ),

            # decrement counter
            do_at(128, MARKER)(dec(1)),
        ),
        # compare
        local(64),
        *(
            link(
                compare(value)()(reject()),
                right(),
            )
            for value in target
        ),
        accept(),
    )(
        reject(),
    ),
)

# 11111111111111111111111111 11111 11111 11 11111111111111111111111111
# from                       symb  write m  to

machine = (transition, 0, (ACCEPT_S, REJECT_S))
# print(execute(tape, machine))

instructions = []
for start, edges in transition.items():
    for symbol, (end, write, move) in edges.items():
        instruction = start

        instruction <<= 5
        instruction |= symbol

        instruction <<= 5
        instruction |= symbol if write is None else write

        instruction <<= 2
        if move == LEFT:
            instruction |= 0b10
        elif move == RIGHT:
            instruction |= 0b01
        else:
            instruction |= 0b00

        instruction <<= 26
        instruction |= end

        instructions.append(instruction)
instructions.sort()

program_c = '''# include "program.h"

const unsigned long program[%d] = {
%s
};
''' % (len(instructions), ',\n'.join(f'    0x{i:016x}' for i in instructions))

program_h = '''# ifndef PROGRAM_H
# define PROGRAM_H

extern const unsigned long program[%d];

# define PROGRAM_LENGTH %d

# define ACCEPT_STATE 0b11111111111111111111111111
# define REJECT_STATE 0b11111111111111111111111110

# define BEGIN_SYMBOL 30
# define END_SYMBOL 31

# endif
''' % (len(instructions), len(instructions))

with open('src/program.c', 'w+') as f:
    f.write(program_c)

with open('src/program.h', 'w+') as f:
    f.write(program_h)
