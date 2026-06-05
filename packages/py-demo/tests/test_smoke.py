"""Smoke tests for the py-demo workspace package."""

from py_demo import greet


def test_greet_default() -> None:
    assert greet() == "hello, luna"


def test_greet_custom_name() -> None:
    assert greet("world") == "hello, world"
