use arrow::array::*;
use arrow_message::prelude::*;

#[derive(Debug, ArrowMessage)]
struct Metadata {
    name: Option<String>,
    width: u32,
    height: u32,
}

#[derive(Debug, ArrowMessage)]
struct Image {
    data: UInt8Array,
    metadata: Option<Metadata>,
}

fn main() -> arrow::error::Result<()> {
    let image = Image {
        data: UInt8Array::from(vec![1, 2, 3]),
        metadata: Some(Metadata {
            name: Some("example".to_string()),
            width: 12,
            height: 12,
        }),
    };

    println!("{:?}", image);

    let arrow = ArrayData::try_from(image)?;
    let flat = arrow.flattened()?;

    let image = Image::try_from(flat)?;
    println!("{:?}", image);

    let arrow = ArrayData::try_from(image)?;
    let (layout, values) = arrow.layout_with_values();
    let arrow = ArrayData::from_layout_and_values(layout, values)?;

    let image = Image::try_from(arrow)?;

    println!("{:?}", image);

    let arrow = ArrayData::try_from(image)?;
    let layout = arrow.layout();

    let size = arrow.required_size();
    let mut target = vec![0u8; size];
    arrow.fill(&mut target);

    let values = arrow::buffer::Buffer::from_vec(target);

    // ...
    // Send those and reconstruct an ArrayData
    // ...

    let arrow = ArrayData::from_layout_and_values(layout, values)?;

    let image = Image::try_from(arrow)?;

    println!("{:?}", image);

    Ok(())
}
