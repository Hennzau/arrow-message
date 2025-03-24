use arrow::array::*;
use arrow_message::prelude::*;

#[derive(Debug)]
enum Encoding {
    RGB8,
    RGBA8,
    BGR8,
    BGRA8,
}

impl Encoding {
    pub fn into_string(self) -> String {
        match self {
            Encoding::RGB8 => "RGB8".to_string(),
            Encoding::RGBA8 => "RGBA8".to_string(),
            Encoding::BGR8 => "BGR8".to_string(),
            Encoding::BGRA8 => "BGRA8".to_string(),
        }
    }

    pub fn try_from_string(value: String) -> arrow::error::Result<Self> {
        match value.as_str() {
            "RGB8" => Ok(Encoding::RGB8),
            "RGBA8" => Ok(Encoding::RGBA8),
            "BGR8" => Ok(Encoding::BGR8),
            "BGRA8" => Ok(Encoding::BGRA8),
            _ => Err(arrow::error::ArrowError::ParseError(format!(
                "Invalid encoding: {}",
                value
            ))),
        }
    }
}

impl ArrowMessage for Encoding {
    fn field(name: impl Into<String>) -> Field {
        String::field(name)
    }

    fn try_from_arrow(data: ArrayData) -> arrow::error::Result<Self> {
        Encoding::try_from_string(String::try_from_arrow(data)?)
    }

    fn try_into_arrow(self) -> arrow::error::Result<ArrayRef> {
        String::try_into_arrow(self.into_string())
    }
}

impl TryFrom<ArrayData> for Encoding {
    type Error = arrow::error::ArrowError;

    fn try_from(data: ArrayData) -> arrow::error::Result<Self> {
        Encoding::try_from_arrow(data)
    }
}

impl TryFrom<Encoding> for ArrayData {
    type Error = arrow::error::ArrowError;

    fn try_from(metadata: Encoding) -> arrow::error::Result<Self> {
        metadata.try_into_arrow().map(|array| array.into_data())
    }
}

#[derive(Debug)]
struct Metadata {
    name: Option<String>,
    width: u32,
    height: u32,
    encoding: Encoding,
}

impl ArrowMessage for Metadata {
    fn field(name: impl Into<String>) -> Field {
        make_union_fields(
            name,
            vec![
                Option::<String>::field("name"),
                Option::<u32>::field("width"),
                Option::<u32>::field("height"),
                Encoding::field("encoding"),
            ],
        )
    }

    fn try_from_arrow(data: arrow::array::ArrayData) -> arrow::error::Result<Self>
    where
        Self: Sized,
    {
        let (map, children) = unpack_union(data);

        Ok(Metadata {
            name: extract_union_data("name", &map, &children)?,
            width: extract_union_data("width", &map, &children)?,
            height: extract_union_data("height", &map, &children)?,
            encoding: extract_union_data("encoding", &map, &children)?,
        })
    }

    fn try_into_arrow(self) -> arrow::error::Result<arrow::array::ArrayRef> {
        let union_fields = get_union_fields::<Self>()?;

        make_union_array(
            union_fields,
            vec![
                self.name.try_into_arrow()?,
                self.width.try_into_arrow()?,
                self.height.try_into_arrow()?,
                self.encoding.try_into_arrow()?,
            ],
        )
    }
}

impl TryFrom<ArrayData> for Metadata {
    type Error = arrow::error::ArrowError;

    fn try_from(data: ArrayData) -> arrow::error::Result<Self> {
        Metadata::try_from_arrow(data)
    }
}

impl TryFrom<Metadata> for ArrayData {
    type Error = arrow::error::ArrowError;

    fn try_from(metadata: Metadata) -> arrow::error::Result<Self> {
        metadata.try_into_arrow().map(|array| array.into_data())
    }
}

#[derive(Debug)]
struct Image {
    data: UInt8Array,
    metadata: Option<Metadata>,
}

impl ArrowMessage for Image {
    fn field(name: impl Into<String>) -> Field {
        make_union_fields(
            name,
            vec![
                UInt8Array::field("data"),
                Option::<Metadata>::field("metadata"),
            ],
        )
    }

    fn try_from_arrow(data: arrow::array::ArrayData) -> arrow::error::Result<Self>
    where
        Self: Sized,
    {
        let (map, children) = unpack_union(data);

        Ok(Image {
            data: extract_union_data("data", &map, &children)?,
            metadata: extract_union_data("metadata", &map, &children)?,
        })
    }

    fn try_into_arrow(self) -> arrow::error::Result<arrow::array::ArrayRef> {
        let union_fields = get_union_fields::<Self>()?;

        make_union_array(
            union_fields,
            vec![self.data.try_into_arrow()?, self.metadata.try_into_arrow()?],
        )
    }
}

impl TryFrom<ArrayData> for Image {
    type Error = arrow::error::ArrowError;

    fn try_from(data: ArrayData) -> arrow::error::Result<Self> {
        Image::try_from_arrow(data)
    }
}

impl TryFrom<Image> for ArrayData {
    type Error = arrow::error::ArrowError;

    fn try_from(image: Image) -> arrow::error::Result<Self> {
        image.try_into_arrow().map(|array| array.into_data())
    }
}

fn main() -> arrow::error::Result<()> {
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

    let arrow = ArrayData::try_from(image)?;
    let image = Image::try_from(arrow)?;

    println!("{:?}", image);

    Ok(())
}
