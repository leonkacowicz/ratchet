"""Python fixture exercising the constructs the native rules touch: functions,
methods, lambdas, every control structure rca scores, word boolean operators,
and Python-specific forms (elif, loop-else, try/except/finally, with, assert,
comprehensions). Used as the Python parity corpus against rust-code-analysis."""


def declared(a, b):
    return a + b


def defaults(a, b=1, *args, **kwargs):
    return a + b


def branches(n):
    if n < 0:
        return "negative"
    elif n == 0:
        return "zero"
    else:
        return "positive"


def loops(items, table):
    for item in items:
        if item:
            continue
    else:
        pass
    n = 0
    while n < 10:
        n += 1
    else:
        pass
    for key in table:
        del table[key]
    return n


def guarded(fn):
    try:
        return fn()
    except ValueError:
        return None
    except (TypeError, KeyError):
        return 0
    finally:
        pass


def resources(path):
    with open(path) as handle:
        assert handle is not None
        return handle.read()


def booleans(a, b, c):
    same = a and b and c
    mixed = a and b or c
    negated = not a and b
    return same or mixed or negated


def conditional(n):
    return "pos" if n > 0 else "neg"


def lambdas(values):
    doubled = map(lambda v: v * 2, values)
    picky = filter(lambda v: v > 0 and v < 10, values)
    return list(doubled) + list(picky)


def nesting(rows):
    for row in rows:
        if row:
            while row:
                if row[0]:
                    return row
    return None


def nested_defs(values):
    def inner(x):
        if x:
            return x
        return 0

    return [inner(v) for v in values]


class Shape:
    def __init__(self, sides):
        self.sides = sides

    def describe(self, prefix):
        if self.sides > 4:
            return prefix + "many"
        return prefix + str(self.sides)

    @staticmethod
    def make(n):
        return Shape(n)
