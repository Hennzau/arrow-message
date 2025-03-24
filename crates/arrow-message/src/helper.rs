use crate::traits::message::ArrowMessage;

pub fn make_union_fields(
    name: impl Into<String>,
    fields: Vec<arrow::datatypes::Field>,
) -> arrow::datatypes::Field {
    arrow::datatypes::Field::new(
        name,
        arrow::datatypes::DataType::Union(
            arrow::datatypes::UnionFields::new(0..fields.len() as i8, fields),
            arrow::datatypes::UnionMode::Dense,
        ),
        false,
    )
}

pub fn unpack_union(
    data: arrow::array::ArrayData,
) -> (
    std::collections::HashMap<String, usize>,
    Vec<arrow::array::ArrayRef>,
) {
    let (fields, _, _, children) = arrow::array::UnionArray::from(data).into_parts();

    let map = fields
        .iter()
        .map(|(id, field)| (field.name().into(), id as usize))
        .collect::<std::collections::HashMap<String, usize>>();

    (map, children)
}

pub fn extract_union_data<T: ArrowMessage>(
    field: &str,
    map: &std::collections::HashMap<String, usize>,
    children: &[arrow::array::ArrayRef],
) -> arrow::error::Result<T> {
    use arrow::array::Array;

    T::try_from_arrow(
        children
            .get(
                *map.get(field)
                    .ok_or(arrow::error::ArrowError::InvalidArgumentError(format!(
                        "Field {} not found",
                        field
                    )))?,
            )
            .ok_or(arrow::error::ArrowError::InvalidArgumentError(format!(
                "Field {} not found",
                field
            )))?
            .into_data(),
    )
}

pub fn get_union_fields<T: ArrowMessage>() -> arrow::error::Result<arrow::datatypes::UnionFields> {
    match T::field("").data_type() {
        arrow::datatypes::DataType::Union(fields, _) => Ok(fields.clone()),
        _ => Err(arrow::error::ArrowError::InvalidArgumentError(
            "Expected Union data type".to_string(),
        )),
    }
}

pub fn make_union_array(
    union_fields: arrow::datatypes::UnionFields,
    children: Vec<arrow::array::ArrayRef>,
) -> Result<arrow::array::ArrayRef, arrow::error::ArrowError> {
    arrow::array::UnionArray::try_new(
        union_fields,
        arrow::buffer::ScalarBuffer::from(vec![]),
        Some(arrow::buffer::ScalarBuffer::from(vec![])),
        children,
    )
    .map(|union| std::sync::Arc::new(union) as arrow::array::ArrayRef)
}
