// C++ fixture exercising the constructs the native rules touch: free functions,
// methods, constructors/destructors, operators, templates, lambdas, every
// control structure rca scores, boolean sequences and goto.
// Used as the C/C++ parity corpus against rust-code-analysis.

#include <vector>
#include <string>

namespace shapes {

int add(int a, int b) {
    return a + b;
}

const char *branches(int n) {
    if (n < 0) {
        return "negative";
    } else if (n == 0) {
        return "zero";
    } else {
        return "positive";
    }
}

int loops(const std::vector<int> &items, int *table, int len) {
    for (size_t i = 0; i < items.size(); i++) {
        if (items[i] > 0) {
            continue;
        }
    }
    for (int value : items) {
        if (value == 0) {
            break;
        }
    }
    int n = 0;
    while (n < 10) {
        n += 1;
    }
    do {
        n -= 1;
    } while (n > 0);
    return n;
}

int switching(int kind) {
    switch (kind) {
        case 1:
            return 1;
        case 2:
            return 2;
        default:
            return 0;
    }
}

int guarded(int value) {
    try {
        if (value < 0) {
            throw std::string("bad");
        }
        return value;
    } catch (const std::string &err) {
        return -1;
    } catch (...) {
        return -2;
    }
}

int jumping(int n) {
    if (n < 0) {
        goto done;
    }
    n += 1;
done:
    return n;
}

int ternary(int n) {
    return n > 0 ? 1 : -1;
}

bool booleans(bool a, bool b, bool c) {
    bool same = a && b && c;
    bool mixed = a && b || c;
    bool negated = !a && b;
    return same || mixed || negated;
}

int lambdas(const std::vector<int> &values) {
    auto doubler = [](int v) { return v * 2; };
    auto guardedLambda = [](int v, int limit) {
        if (v > limit) {
            return v;
        }
        return 0;
    };
    return doubler(1) + guardedLambda(2, 3);
}

int nesting(const std::vector<std::vector<int>> &rows) {
    for (const auto &row : rows) {
        if (!row.empty()) {
            for (int cell : row) {
                if (cell > 0) {
                    return cell;
                }
            }
        }
    }
    return 0;
}

class Shape {
public:
    Shape(int sides) : sides_(sides) {}
    ~Shape() {}

    int sides() const {
        return sides_;
    }

    bool operator==(const Shape &other) const {
        return sides_ == other.sides_;
    }

private:
    int sides_;
};

template <typename T>
T largest(const std::vector<T> &values) {
    T best = values[0];
    for (const T &v : values) {
        if (v > best) {
            best = v;
        }
    }
    return best;
}

}  // namespace shapes
