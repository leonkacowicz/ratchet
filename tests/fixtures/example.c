/* Minimal C fixture: a couple of functions with real control flow so the
 * structural collector has metrics (cyclomatic/cognitive) to measure. */
#include <stddef.h>

int add(int a, int b) {
    return a + b;
}

int classify(int n) {
    if (n < 0) {
        return -1;
    } else if (n == 0) {
        return 0;
    }
    return 1;
}
