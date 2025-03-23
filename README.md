# arrow-message

`arrow-message` makes it possible to create a Message struct in Rust or Python and convert it into a single `arrow::array::ArrayData` without any copy. It's also possible to get back to the initial struct without any copy as well.

The resulting `arrow::array::ArrayData` can then be sent safely over the network, a `mpsc` channel or to a Python script thanks to the `pyo3` crate and the `pyarrow` feature.

The project aims to be used in context where we want to send a single payload containing multiple large fields. Like a struct representing an image or a video frame. This is ideal for Robotics and AI applications.

# Example

```Rust
use arrow::array::*;
use arrow_message::prelude::*;

#[derive(Debug, ArrowMessage)]
enum Encoding {
    RGB8,
    RGBA8,
    BGR8,
    BGRA8,
}

#[derive(Debug, ArrowMessage)]
struct Metadata {
    name: Option<String>,
    width: u32,
    height: u32,
    encoding: Encoding,
}

#[derive(Debug, ArrowMessage)]
struct Image {
    data: UInt8Array,
    metadata: Option<Metadata>,
}

fn main() -> Result<()> {
    use miette::IntoDiagnostic;

    let image = Image {
        data: UInt8Array::from(vec![1, 2, 3]),
        metadata: Some(Metadata {
            name: Some("example".to_string()),
            width: 12,
            height: 12,
            encoding: Encoding::RGB8,
        }),
    };

    println!("{:?}", image);

    let arrow = ArrayData::try_from(image).into_diagnostic()?;
    let image = Image::try_from(arrow).into_diagnostic()?;

    println!("{:?}", image);

    Ok(())
}
```

You can see an expanded version without the Derive macro [here](crates/arrow-message/examples/enum_impl.rs), and a more complex example [here](crates/arrow-message/examples/complex.rs).

A python version [here](crates/arrow-message-python/examples/enum_inherit.py)

```python
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
```

## Just run examples

We use a `justfile` to run examples:

```bash
just example-derive-enum # rust enum_derive.rs example
just example-derive-inherit # python enum_inherit.py example
```

# Features

- [x] Fields supported
  - [x] Primitive types
  - [x] Optional Primitive Rust types
  - [x] Arrow Arrays for Rust, Numpy Arrays for Python
  - [x] Optional Arrow Arrays for Rust, Numpy Arrays for Python
  - [x] Rust structs that implement ArrowMessage, Python dataclasses that inherit from ArrowMessage for Python
  - [x] Rust simple enums that implement ArrowMessage for Rust, Python simple enums that inherit from ArrowMessage for Python
  - [x] Optional structs/classes that implement/inherit ArrowMessage
  - [x] Optional enums that implement/inherit ArrowMessage
  - [?] ~Enums with variant that implements/inherit ArrowMessage? I don't think it's possible, as an ArrowMessage should know it's exact datatype layout at compile time (only Option that are represented as NullArray when on runtime the value is None)~

- [ ] Operations supported
  - [x] Into/From ArrayData
  - [ ] ArrayData Flattening

# What's Next?

- [ ] Think about Vec/List support. is it possible and is it relevant?.
- [ ] Improved error handling and validation: too much panic! in arrow that we must catch.
- [ ] Make to python API fully Rust with PyO3 (may be hard because we use a lot of python runtime tricks)
- [ ] Enhanced documentation and examples
- [ ] Integration with other libraries and frameworks
