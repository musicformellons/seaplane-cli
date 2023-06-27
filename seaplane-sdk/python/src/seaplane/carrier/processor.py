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
        processor.flush()  # Flush the messages.

"""
from io import BufferedReader, BufferedWriter
import logging
import os
import threading
import time
from typing import List, Optional


class ProcessorIO:
    """ProcessorIO handles IO to a stdpipe with a framed transport.

    ProcessorIO provides three thread-safe methods, `read`, `write` and `flush`.

    `read` reads a single message from `input_fifo`.

    `write` appends a message to be written out, to a `messages` buffer.

    `flush` flushes the messages buffer to the `output_fifo`.
    """

    def __init__(self, input_fifo_path: str, output_fifo_path: str) -> None:
        self._input_fifo_path = input_fifo_path
        self._output_fifo_path = output_fifo_path
        self._input_fifo: Optional[BufferedReader] = None
        self._output_fifo: Optional[BufferedWriter] = None
        self._read_lock = threading.Lock()
        self._write_lock = threading.Lock()
        self._messages: List[bytes] = []

    def read(self) -> bytes:
        """
        `read` takes a single message, the frame format is a 4 byte header
        (unsigned 32 bit int, in big-endian order) indicating the frame size,
        followed by the frame itself.

        Example: ```b'\\x00\\x00\\x00\\x05hello'```
        """
        with self._read_lock:
            if self._input_fifo is not None:
                # Read frame header.
                header = self._input_fifo.read(4)
                frame_size = int.from_bytes(header, "big", signed=False)
                return self._input_fifo.read(frame_size)

        return bytes(0)

    def write(self, msg: bytes) -> None:
        """
        `write` appends a message to the message buffer.

        Once all the messages are written, they need to be `flush`ed.
        """
        self._messages.append(msg)

    def flush(self) -> None:
        """
        `flush` flushes the messages buffer to the output_fifo, using a format:
        - 4 byte header (unsigned 32 bit int) with the frame size
        - 4 byte header (unsigned 32 bit int) with the number of messages
        - all the messages in the frame format:
          - 4 byte header (unsigned 32 bit int) with the message size
          - message bytes

        Example:

        2 messages: `hello` and `world` are encoded as follows:

        ```
        frame_size = b'\\x00\\x00\\x00\\x16' # 22 bytes that follow
        number_of_messages = b'\\x00\\x00\\x00\\x02' # 2 messages
        message1 = b'\\x00\\x00\\x00\\x05hello'
        message2 = b'\\x00\\x00\\x00\\x05world'
        ```

        Which results in this message being flushed to fifo_output:

        ```
        b'\\x00\\x00\\x00\\x16\\x00\\x00\\x00\\x02\\x00\\x00\\x00\\x05hello\\x00\\x00\\x00\\x05world'
        ```

        """
        with self._write_lock:
            output = bytearray()
            num_msgs = len(self._messages).to_bytes(4, "big")
            output.extend(num_msgs)
            for msg in self._messages:
                msg_len = len(msg).to_bytes(4, "big")
                output.extend(msg_len + msg)
            frame_size = len(output).to_bytes(4, "big")
            if self._output_fifo:
                self._output_fifo.write(frame_size + output)
                self._output_fifo.flush()
            self._messages.clear()

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
