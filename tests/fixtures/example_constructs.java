// Java fixture exercising the constructs the native rules touch: methods,
// constructors, lambdas, every control structure rca scores, boolean sequences,
// and Java forms (enhanced for, switch, try/catch/finally, ternary).
// Used as the Java parity corpus against rust-code-analysis.

import java.util.List;
import java.util.function.Function;

public class Shapes {

    private final int sides;

    public Shapes(int sides) {
        this.sides = sides;
    }

    public Shapes(int sides, String name) {
        this.sides = sides;
    }

    public int getSides() {
        return sides;
    }

    public String branches(int n) {
        if (n < 0) {
            return "negative";
        } else if (n == 0) {
            return "zero";
        } else {
            return "positive";
        }
    }

    public int loops(List<Integer> items, int[] table) {
        for (int i = 0; i < items.size(); i++) {
            if (items.get(i) > 0) {
                continue;
            }
        }
        for (int value : table) {
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

    public int switching(String kind) {
        switch (kind) {
            case "a":
                return 1;
            case "b":
                return 2;
            default:
                return 0;
        }
    }

    public Integer guarded(Function<Integer, Integer> fn) {
        try {
            return fn.apply(1);
        } catch (IllegalArgumentException err) {
            return null;
        } catch (RuntimeException err) {
            return 0;
        } finally {
            System.out.println("done");
        }
    }

    public String ternary(int n) {
        return n > 0 ? "pos" : "neg";
    }

    public boolean booleans(boolean a, boolean b, boolean c) {
        boolean same = a && b && c;
        boolean mixed = a && b || c;
        boolean negated = !a && b;
        return same || mixed || negated;
    }

    public Function<Integer, Integer> lambdas(List<Integer> values) {
        Function<Integer, Integer> doubler = v -> v * 2;
        Function<Integer, Integer> guardedLambda = v -> {
            if (v > 0) {
                return v;
            }
            return 0;
        };
        return doubler.andThen(guardedLambda);
    }

    public Integer nesting(List<List<Integer>> rows) {
        for (List<Integer> row : rows) {
            if (row != null) {
                for (Integer cell : row) {
                    if (cell > 0) {
                        return cell;
                    }
                }
            }
        }
        return null;
    }

    public static Shapes make(int n) {
        return new Shapes(n);
    }
}
