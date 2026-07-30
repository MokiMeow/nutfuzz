#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

int main(void) {
    uint8_t input[4096];
    const size_t length = fread(input, 1, sizeof(input), stdin);

    if (length < 7 || input[0] != 'F' || input[1] != 'U' ||
        input[2] != 'Z' || input[3] != 'Z' || input[4] != 0x02 ||
        input[5] != 0x42) {
        return 0;
    }
    if (input[6] > 200) {
        abort();
    }
    return 0;
}
