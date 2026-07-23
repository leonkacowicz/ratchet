// Minimal Java fixture: a class with a couple of methods, one with branching,
// so the structural collector finds named functions and real metrics.
class Example {
    int add(int a, int b) {
        return a + b;
    }

    String classify(int n) {
        if (n < 0) {
            return "negative";
        } else if (n == 0) {
            return "zero";
        }
        return "positive";
    }
}
