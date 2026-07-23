"""Minimal Python fixture: functions with branching so the structural
collector has cyclomatic/cognitive metrics to measure."""


def add(a, b):
    return a + b


def classify(n):
    if n < 0:
        return "negative"
    elif n == 0:
        return "zero"
    return "positive"
