use std::sync::Arc;

use arrow::{
    array::{
        Array, ArrayData, ArrayRef, ArrowPrimitiveType, BooleanArray, Float32Array, Float64Array,
        Int8Array, Int16Array, Int32Array, Int64Array, NullArray, PrimitiveArray, StringArray,
        UInt8Array, UInt16Array, UInt32Array, UInt64Array,
    },
    datatypes::{DataType, Field},
    error::ArrowError,
};

pub trait ArrowMessage {
    fn field(name: impl Into<String>) -> Field;

    fn try_into_arrow(self) -> miette::Result<ArrayRef, ArrowError>;
    fn try_from_arrow(data: ArrayData) -> miette::Result<Self, ArrowError>
    where
        Self: Sized;
}

impl<T> ArrowMessage for Option<T>
where
    T: ArrowMessage,
{
    fn field(name: impl Into<String>) -> Field {
        T::field(name).with_nullable(true)
    }

    fn try_into_arrow(self) -> miette::Result<ArrayRef, ArrowError> {
        match self {
            Some(value) => value.try_into_arrow(),
            None => Ok(Arc::new(NullArray::new(0)) as ArrayRef),
        }
    }

    fn try_from_arrow(data: ArrayData) -> miette::Result<Self, ArrowError>
    where
        Self: Sized,
    {
        match data.data_type() {
            DataType::Null => Ok(None),
            _ => T::try_from_arrow(data).map(|value| Some(value)),
        }
    }
}

impl<T> ArrowMessage for PrimitiveArray<T>
where
    T: ArrowPrimitiveType,
{
    fn field(name: impl Into<String>) -> Field {
        Field::new(name, T::DATA_TYPE, false)
    }

    fn try_into_arrow(self) -> miette::Result<ArrayRef, ArrowError> {
        Ok(Arc::new(self) as ArrayRef)
    }

    fn try_from_arrow(data: ArrayData) -> miette::Result<Self, ArrowError>
    where
        Self: Sized,
    {
        Ok(PrimitiveArray::from(data))
    }
}

macro_rules! impl_arrow_field {
    ($type:ty, $data_type:expr, $array_type:ty) => {
        impl ArrowMessage for $type {
            fn field(name: impl Into<String>) -> Field {
                Field::new(name, $data_type, false)
            }

            fn try_into_arrow(self) -> miette::Result<ArrayRef, ArrowError> {
                Ok(Arc::new(<$array_type>::from(vec![self])) as ArrayRef)
            }

            fn try_from_arrow(data: ArrayData) -> miette::Result<Self, ArrowError>
            where
                Self: Sized,
            {
                let array = <$array_type>::from(data);
                match array.len() {
                    0 => Err(ArrowError::InvalidArgumentError(
                        "Array is empty".to_string(),
                    )),
                    _ => Ok(array.value(0)),
                }
            }
        }
    };
}

impl_arrow_field!(u8, DataType::UInt8, UInt8Array);
impl_arrow_field!(u16, DataType::UInt16, UInt16Array);
impl_arrow_field!(u32, DataType::UInt32, UInt32Array);
impl_arrow_field!(u64, DataType::UInt64, UInt64Array);
impl_arrow_field!(i8, DataType::Int8, Int8Array);
impl_arrow_field!(i16, DataType::Int16, Int16Array);
impl_arrow_field!(i32, DataType::Int32, Int32Array);
impl_arrow_field!(i64, DataType::Int64, Int64Array);
impl_arrow_field!(f32, DataType::Float32, Float32Array);
impl_arrow_field!(f64, DataType::Float64, Float64Array);
impl_arrow_field!(bool, DataType::Boolean, BooleanArray);

impl ArrowMessage for String {
    fn field(name: impl Into<String>) -> Field {
        Field::new(name, DataType::Utf8, false)
    }

    fn try_into_arrow(self) -> miette::Result<ArrayRef, ArrowError> {
        Ok(Arc::new(StringArray::from(vec![self])) as ArrayRef)
    }

    fn try_from_arrow(data: ArrayData) -> miette::Result<Self, ArrowError>
    where
        Self: Sized,
    {
        let array = StringArray::from(data);
        match array.len() {
            0 => Err(ArrowError::InvalidArgumentError(
                "Array is empty".to_string(),
            )),
            _ => Ok(array.value(0).into()),
        }
    }
}

impl ArrowMessage for StringArray {
    fn field(name: impl Into<String>) -> Field {
        Field::new(name, DataType::Utf8, false)
    }

    fn try_into_arrow(self) -> miette::Result<ArrayRef, ArrowError> {
        Ok(Arc::new(self) as ArrayRef)
    }

    fn try_from_arrow(data: ArrayData) -> miette::Result<Self, ArrowError>
    where
        Self: Sized,
    {
        Ok(StringArray::from(data))
    }
}
