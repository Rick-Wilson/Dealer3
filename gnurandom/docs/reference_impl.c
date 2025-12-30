#include <stdio.h>
#include <stdint.h>

#define DEG_3 31
#define SEP_3 3

// Simulate exactly what the x86_64 __random.o does with 64-bit long
int64_t state[31];
int64_t *fptr;
int64_t *rptr;
int rand_deg = DEG_3;

void my_srandom_64(unsigned int x) {
    state[0] = x;

    // LCG initialization - exactly as in x86_64 code
    for (int i = 1; i < rand_deg; i++) {
        state[i] = state[i-1] * 1103515145LL + 12345LL;
    }

    fptr = &state[SEP_3];
    rptr = &state[0];

    // Warmup
    for (int i = 0; i < 10 * rand_deg; i++) {
        *fptr += *rptr;
        ++fptr;
        if (fptr >= &state[rand_deg]) {
            fptr = state;
            ++rptr;
        } else {
            ++rptr;
            if (rptr >= &state[rand_deg])
                rptr = state;
        }
    }
}

uint32_t my_random_64(void) {
    // Add (64-bit)
    *fptr += *rptr;

    // Shift and mask - exactly as in x86_64 code
    // sarq $1, %rax  (64-bit arithmetic shift)
    // and with 0x7fffffffffffffff
    int64_t result = (*fptr >> 1) & 0x7fffffffffffffffLL;

    // Advance pointers
    ++fptr;
    if (fptr >= &state[rand_deg]) {
        fptr = state;
        ++rptr;
    } else {
        ++rptr;
        if (rptr >= &state[rand_deg])
            rptr = state;
    }

    // Return as uint32_t (truncate)
    return (uint32_t)result;
}

int main() {
    my_srandom_64(1);

    printf("Testing 64-bit implementation:\n");
    for (int i = 0; i < 20; i++) {
        printf("%u\n", my_random_64());
    }

    return 0;
}
