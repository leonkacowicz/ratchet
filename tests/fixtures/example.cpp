// Minimal C++ fixture: a class with a method plus a free function, enough for
// the structural collector to find named functions and compute metrics.
#include <vector>

struct Accumulator {
    int total = 0;

    void add(int value) {
        total += value;
    }
};

int sum(const std::vector<int> &values) {
    int total = 0;
    for (int value : values) {
        total += value;
    }
    return total;
}
