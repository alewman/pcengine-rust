#!/usr/bin/env python3
"""Two places where pcengine-python v0.1.0's behaviour looked unintended while it was being
transcribed, shown on the reference itself (run on its interpreter). Neither is a divergence:
the Rust board does the same (README, "Notes on the reference").

    external/pcengine-python/.venv-pypy/bin/python scripts/reference_notes.py
"""

from pcengine_python.board import Board
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


def a_waited_dma_never_meets_the_display() -> None:
    """`Vdc._data_access` runs the VDC up to the transfer's end, lines ahead of the board's
    line clock, so a transfer the CPU waits on completes as if the display never started."""
    for waits in (False, True):
        board = Board(dma_resumes=False)
        base = 0x1FE000

        def reg(register: int, value: int, board=board) -> None:
            board.write_byte(base, register)
            board.write_byte(base + 2, value & 0xFF)
            board.write_byte(base + 3, value >> 8)

        for register, value in ((0x0A, 0x0202), (0x0B, 0x041F), (0x0C, 0x0F02), (0x0D, 0x00EF), (0x0E, 3), (0x05, 0x0080)):
            reg(register, value)
        board.advance(1365 * (262 + 259 + 5) - board.mclk)  # five lines into the second frame's sync
        reg(0x10, 0x1000)
        reg(0x11, 0x3000)
        reg(0x12, 0x3FFF)  # 16,384 words: 96 lines at 5 MHz; the display starts in 15
        if waits:
            reg(0x01, 0x0000)
            board.read_byte(base + 2)  # a VRAM read: the CPU waits for the transfer
        board.advance(1365 * 40)
        left = (board.vdc.regs.value[0x12] + 1) & 0xFFFF
        print(f"dma_resumes=False, a 16,384-word DMA started 15 lines before the display, "
              f"{'then a VRAM read' if waits else 'left alone'}: {left} words not transferred")  # fmt: skip


if __name__ == "__main__":
    a_waited_dma_never_meets_the_display()
    silent_units_after_construction()
