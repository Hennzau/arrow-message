from pyarrow_message import ArrowMessage
from dataclasses import dataclass
from typing import Optional
from enum import Enum

import numpy as np


class Encoding(ArrowMessage, Enum):
    RGB8 = "RGB8"
    RGBA8 = "RGBA8"
    BGR8 = "BGR8"
    BGRA8 = "BGRA8"


@dataclass
class Metadata(ArrowMessage):
    name: Optional[str]
    width: np.uint32
    height: np.uint32
    encoding: Optional[Encoding]


@dataclass
class Image(ArrowMessage):
    data: np.ndarray
    metadata: Optional[Metadata]


def main():
    image = Image(
        data=np.array([1, 2, 3], dtype=np.uint8),
        metadata=Metadata(
            width=np.uint32(12),
            height=np.uint32(12),
            name="example",
            encoding=Encoding.RGB8,
        ),
    )

    print(image)
    arrow = image.to_arrow()
    image2 = Image.from_arrow(arrow)

    print(image2)


if __name__ == "__main__":
    main()
