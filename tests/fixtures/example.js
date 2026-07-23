// Minimal JavaScript fixture: functions with branching so the structural
// collector has metrics to measure.
function add(a, b) {
  return a + b;
}

function classify(n) {
  if (n < 0) {
    return "negative";
  } else if (n === 0) {
    return "zero";
  }
  return "positive";
}
