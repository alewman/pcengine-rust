#!/usr/bin/env python3
"""A place where pcengine-python v0.1.0's behaviour looked unintended while it was being
transcribed, shown on the reference itself (run on its interpreter). It is not a divergence:
the Rust board gives the same samples (README, "Notes on the reference").

    external/pcengine-python/.venv-pypy/bin/python scripts/reference_notes.py
"""

from pcengine_python.psg import Psg


def silent_units_after_construction() -> None:
    """`Psg._levels` is built once from `silent_units`; scripts/psg_compare.py applies VARIANTS
    with setattr on an instance, which leaves the host-rate table as it was."""
    late = Psg()
    late.silent_units = 31
    Psg.silent_units = 31
    early = Psg()
    Psg.silent_units = 30
    print(f"silent_units = 31 set on an instance: level of 30 units {late._levels[30]!r}; "
          f"set before construction: {early._levels[30]!r}")  # fmt: skip


if __name__ == "__main__":
    silent_units_after_construction()
