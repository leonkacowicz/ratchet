// TypeScript fixture exercising the constructs the JS-family rules touch, plus
// TypeScript-only syntax (types, interfaces, generics, enums) that must not
// perturb the metrics. Used as the TS parity corpus against rust-code-analysis.

interface Row {
  id: number;
  name: string;
  hidden?: boolean;
}

type Handler = (row: Row) => string;

enum Kind {
  A = "a",
  B = "b",
}

function declared(a: number, b: number): number {
  return a + b;
}

const expression = function (a: number, b: number, c: number): number {
  return a + b + c;
};

const arrow = (x: number): number => x * 2;

const arrowBlock = (x: number, y: number): number => {
  return x + y;
};

function* generator(n: number): Generator<number> {
  yield n;
}

class Table<T extends Row> {
  private rows: T[];

  constructor(rows: T[]) {
    this.rows = rows;
  }

  find(id: number): T | null {
    for (const row of this.rows) {
      if (row.id === id) {
        return row;
      }
    }
    return null;
  }

  static empty<U extends Row>(): Table<U> {
    return new Table<U>([]);
  }
}

function branches(n: number): string {
  if (n < 0) {
    return "negative";
  } else if (n === 0) {
    return "zero";
  } else {
    return "positive";
  }
}

function loops(items: number[], table: Record<string, number>): number {
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

function switching(kind: Kind): number {
  switch (kind) {
    case Kind.A:
      return 1;
    case Kind.B:
      return 2;
    default:
      return 0;
  }
}

function guarded(fn: () => number): number | null {
  try {
    return fn();
  } catch (err) {
    return null;
  }
}

function booleans(a: boolean, b: boolean, c: boolean): boolean {
  const same = a && b && c;
  const mixed = (a && b) || c;
  const negated = !a && b;
  return same || mixed || negated;
}

function nesting(rows: Row[][]): Row | null {
  for (const group of rows) {
    if (group) {
      for (const row of group) {
        if (!row.hidden) {
          return row;
        }
      }
    }
  }
  return null;
}

function withClosure(values: number[], handler: Handler): number[] {
  const mapped = values.map((v) => {
    if (v > 0) {
      return v;
    }
    return 0;
  });

  function inner(x: number): number {
    return x > 0 ? x : 0;
  }

  return mapped.concat(inner(1));
}
