example-derive:
    cargo run -p arrow-message --example derive

example-enum-derive:
    cargo run -p arrow-message --example enum_derive

example-impl:
    cargo run -p arrow-message --example impl

example-enum-impl:
    cargo run -p arrow-message --example enum_impl

example-complex:
    cargo run -p arrow-message --example complex

python-build:
    uv sync --directory crates/arrow-message-python

python-dev:
    uv sync --directory crates/arrow-message-python --extra tests

example-enum-inherit:
    uv run --directory crates/arrow-message-python examples/enum_inherit.py

format:
    uv run --directory crates/arrow-message-python ruff format
