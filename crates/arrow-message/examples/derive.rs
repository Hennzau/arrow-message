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

fn main() -> Result<()> {
    use miette::IntoDiagnostic;

    let image = Image {
        data: UInt8Array::from(vec![1, 2, 3]),
        metadata: Some(Metadata {
            name: Some("example".to_string()),
            width: 12,
            height: 12,
        }),
    };

    println!("{:?}", image);

    let arrow = ArrayData::try_from(image).into_diagnostic()?;
    let image = Image::try_from(arrow).into_diagnostic()?;

    println!("{:?}", image);

    Ok(())
}
