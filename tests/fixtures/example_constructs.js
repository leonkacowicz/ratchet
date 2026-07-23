// JavaScript fixture exercising the constructs the native metric rules touch:
// every function-space kind, each control structure rca scores, boolean
// sequences, and nesting. Used as the JS parity corpus against
// rust-code-analysis.

// --- function-space kinds -------------------------------------------------

function declared(a, b) {
  return a + b;
}

const expression = function (a, b, c) {
  return a + b + c;
};

const arrow = (x) => x * 2;

const arrowBlock = (x, y) => {
  return x + y;
};

function* generator(n) {
  yield n;
}

class Shape {
  constructor(sides) {
    this.sides = sides;
  }

  describe(prefix) {
    return prefix + this.sides;
  }

  static make(n) {
    return new Shape(n);
  }
}

// --- control structures ---------------------------------------------------

function branches(n) {
  if (n < 0) {
    return "negative";
  } else if (n === 0) {
    return "zero";
  } else {
    return "positive";
  }
}

function loops(items, table) {
  for (let i = 0; i < items.length; i++) {
    if (items[i]) {
      continue;
    }
  }
  for (const key in table) {
    delete table[key];
  }
  let n = 0;
  while (n < 10) {
    n += 1;
  }
  do {
    n -= 1;
  } while (n > 0);
  return n;
}

function switching(kind) {
  switch (kind) {
    case "a":
      return 1;
    case "b":
      return 2;
    default:
      return 0;
  }
}

function guarded(fn) {
  try {
    return fn();
  } catch (err) {
    return null;
  }
}

function ternary(n) {
  return n > 0 ? "pos" : "neg";
}

// --- boolean sequences ----------------------------------------------------

function booleans(a, b, c) {
  const same = a && b && c;
  const mixed = a && b || c;
  const negated = !a && b;
  return same || mixed || negated;
}

// --- nesting, closures, nested functions ----------------------------------

function nesting(rows) {
  for (const row of rows) {
    if (row) {
      while (row.next) {
        if (row.next.done) {
          return row;
        }
      }
    }
  }
  return null;
}

function withClosure(values) {
  const mapped = values.map((v) => {
    if (v > 0) {
      return v;
    }
    return 0;
  });

  function inner(x) {
    if (x) {
      return x;
    }
    return 0;
  }

  return mapped.concat(inner(1));
}

// --- labeled jumps --------------------------------------------------------

function labeled(grid) {
  outer: for (const row of grid) {
    for (const cell of row) {
      if (cell) {
        break outer;
      }
      continue outer;
    }
  }
  return grid;
}
