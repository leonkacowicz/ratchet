// Minimal TSX fixture: a component plus a helper with branching, exercising
// JSX inside TypeScript.
function badge(n: number): string {
  if (n < 0) {
    return "negative";
  } else if (n === 0) {
    return "zero";
  }
  return "positive";
}

const Badge = ({ n }: { n: number }) => <span>{badge(n)}</span>;
