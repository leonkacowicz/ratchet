// Minimal TypeScript fixture: functions with branching so the structural
// collector has metrics to measure.
function add(a: number, b: number): number {
  return a + b;
}

function classify(n: number): string {
  if (n < 0) {
    return "negative";
  } else if (n === 0) {
    return "zero";
  }
  return "positive";
}
