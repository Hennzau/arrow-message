use arrow::array::*;
use arrow_message::prelude::*;

#[derive(Debug, ArrowMessage)]
struct Header {
    timestamp: u64,
}

#[derive(Debug, ArrowMessage)]
struct Metadata {
    width: u32,
    height: u32,

    name: Option<String>,
    header: Option<Header>,
}

#[derive(Debug, ArrowMessage)]
struct Output {
    first_array: UInt8Array,
    second_array: Float32Array,
    third_array: Option<Int32Array>,

    metadata: Metadata,
}

fn main() -> arrow::error::Result<()> {
    let output = Output {
        first_array: UInt8Array::from(vec![1, 2, 3]),
        second_array: Float32Array::from(vec![4.0, 5.0, 6.0]),
        third_array: Some(Int32Array::from(vec![7, 8, 9])),
        metadata: Metadata {
            width: 10,
            height: 20,

            name: Some(String::from("Hello!")),
            header: Some(Header {
                timestamp: 1234567890,
            }),
        },
    };

    println!("{:?}", output);

    let arrow = ArrayData::try_from(output)?;
    println!("{:?}", arrow);

    let output = Output::try_from(arrow)?;
    println!("{:?}", output);

    Ok(())
}
