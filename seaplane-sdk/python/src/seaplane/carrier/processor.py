"""process module hooks up to input/output FIFOs.

The default FIFOs are input.fifo (read from) and output.fifo (write
to). These can be overridden with `SEAPLANE_INPUT_FIFO` and
`SEAPLANE_OUTPUT_FIFO` environment variables respectively.

Usage::

    from seaplane.carrier import processor

    # Reverse all messages.
    while True:
        msg = processor.read()  # Read an entire message (bytes).
        processor.write(msg[::-1])  # Write the reversed bytes.

"""
from io import BufferedReader, BufferedWriter
import logging
import os
import threading
import time
from typing import Any, Optional


class ProcessorIO:
    """ProcessorIO handles IO to a stdpipe with a framed transport.

    ProcessorIO provides two thread-safe methods, `read` and
    `write`. The frame format is a 4 byte header (unsigned 32 bit int,
    in big-endian order) indicating the frame size, followed by the
    frame itself.
    """

    def __init__(self, input_fifo_path: str, output_fifo_path: str) -> None:
        self._input_fifo_path = input_fifo_path
        self._output_fifo_path = output_fifo_path
        self._input_fifo: Optional[BufferedReader] = None
        self._output_fifo: Optional[BufferedWriter] = None
        self._read_lock = threading.Lock()
        self._write_lock = threading.Lock()

    def read(self) -> Any:
        """Read an entire framed message."""
        with self._read_lock:
            # Read frame header.
            if self._input_fifo is not None:
                header = self._input_fifo.read(4)
                frame_size = int.from_bytes(header, "big", signed=False)
                return self._input_fifo.read(frame_size)

    def write(self, msg: bytes) -> None:
        """Frame and write a message."""
        with self._write_lock:
            frame_size = len(msg)
            header = frame_size.to_bytes(4, "big")
            if self._output_fifo is not None:
                self._output_fifo.write(header + msg)
                self._output_fifo.flush()

    def start(self) -> None:
        logging.info("opening input {}".format(self._input_fifo_path))
        logging.info("opening output {}".format(self._output_fifo_path))
        for _ in range(10):
            try:
                self._input_fifo = open(self._input_fifo_path, "rb")
                self._output_fifo = open(self._output_fifo_path, "wb")
            except FileNotFoundError as exc:
                logging.error(exc)
                time.sleep(2)
            else:
                return
        raise FileNotFoundError("input/output")


processor = ProcessorIO(
    os.getenv("SEAPLANE_INPUT_FIFO", "input.fifo"),
    os.getenv("SEAPLANE_OUTPUT_FIFO", "output.fifo"),
)
