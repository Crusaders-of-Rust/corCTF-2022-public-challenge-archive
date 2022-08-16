

flag = b'corctf{oops_all_p1p3lin3d..}'
print(flag)

inside = flag[7:-1]
inside = list(inside)

bits = []
for x in inside:
    bits += [(x >> k) & 1 for k in range(7)]

vals = []
for i in range(0,len(bits),10):
    s = bits[i:i+10]
    v = int(''.join([str(x) for x in s[::-1]]), 2)
    vals.append(v)

print(vals)

curr = 0x1337
bump = 0x4321
for j in range(5):
    for i in range(14):
        vals[i] = ((vals[i] ^ 0xbeef) + curr) & 0x1fff
        vals[i] ^= 0xcafe

        curr = ((curr + bump) ^ 0xaaaa) & 0xffff

print('magic', vals)

bits = []
for x in vals:
    bits += [(x >> k) & 1 for k in range(13)]

print(''.join([str(x) for x in bits]))


# ----

from z3 import *

def solve(target):
    initial = [BitVec('b%d' % i, 13) for i in range(14)]
    vals = [x for x in initial]

    curr = 0x1337
    bump = 0x4321
    for j in range(5):
        for i in range(14):
            vals[i] = ((vals[i] ^ 0xbeef) + curr)
            vals[i] ^= 0xcafe

            curr = ((curr + bump) ^ 0xaaaa) & 0xffff

    s = Solver()
    for i in range(14):
        s.add(vals[i] == target[i])

    print(s.check())
    m = s.model()
    
    out = [m[x].as_long() for x in initial]

    bits = []
    for x in out:
        bits += [(x >> k) & 1 for k in range(10)]

    vals = []
    for i in range(0,len(bits),7):
        s = bits[i:i+7]
        v = int(''.join([str(x) for x in s[::-1]]), 2)
        vals.append(v)

    return bytes(vals)


print(solve(vals))

