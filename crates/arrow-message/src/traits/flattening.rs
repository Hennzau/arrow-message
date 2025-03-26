use arrow::{
    array::{ArrayData, BufferSpec},
    buffer::{Buffer, MutableBuffer},
    datatypes::DataType,
    error::Result,
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BufferOffset {
    pub len: usize,
    pub offset: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArrayDataLayout {
    pub data_type: DataType,
    pub len: usize,
    pub null_bit_buffer: Option<Vec<u8>>,
    pub offset: usize,
    pub buffers: Vec<BufferOffset>,
    pub child_data: Vec<ArrayDataLayout>,
}

pub trait ArrayDataFlattening {
    fn layout_with_values(&self) -> (ArrayDataLayout, Buffer);
    fn flattened(&self) -> Result<ArrayData>;

    fn layout(&self) -> ArrayDataLayout;
    fn required_size(&self) -> usize;
    fn fill(&self, target: &mut [u8]);

    fn from_layout_and_values(layout: ArrayDataLayout, values: Buffer) -> Result<ArrayData>;
}

impl ArrayDataFlattening for ArrayData {
    fn layout(&self) -> ArrayDataLayout {
        fn layout_inner(array: &ArrayData, next_offset: &mut usize) -> ArrayDataLayout {
            let mut buffers = Vec::new();
            let mut child_data = Vec::new();

            let layout = arrow::array::layout(array.data_type());

            for (buffer, spec) in array.buffers().iter().zip(&layout.buffers) {
                if let BufferSpec::FixedWidth { alignment, .. } = spec {
                    *next_offset = (*next_offset).div_ceil(*alignment) * alignment;
                }

                buffers.push(BufferOffset {
                    len: buffer.len(),
                    offset: *next_offset,
                });

                *next_offset += buffer.len();
            }

            for child in array.child_data() {
                child_data.push(layout_inner(child, next_offset));
            }

            ArrayDataLayout {
                data_type: array.data_type().clone(),
                len: array.len(),
                null_bit_buffer: array.nulls().map(|b| b.validity().to_owned()),
                offset: array.offset(),
                buffers,
                child_data,
            }
        }

        let mut next_offset = 0;

        layout_inner(self, &mut next_offset)
    }

    fn required_size(&self) -> usize {
        fn required_size_inner(array: &ArrayData, next_offset: &mut usize) {
            let layout = arrow::array::layout(array.data_type());

            for (buffer, spec) in array.buffers().iter().zip(&layout.buffers) {
                if let BufferSpec::FixedWidth { alignment, .. } = spec {
                    *next_offset = (*next_offset).div_ceil(*alignment) * alignment;
                }

                *next_offset += buffer.len();
            }

            for child in array.child_data() {
                required_size_inner(child, next_offset);
            }
        }

        let mut next_offset = 0;
        required_size_inner(self, &mut next_offset);

        next_offset
    }

    fn fill(&self, target: &mut [u8]) {
        fn fill_inner(array: &ArrayData, next_offset: &mut usize, target: &mut [u8]) {
            let layout = arrow::array::layout(array.data_type());

            for (buffer, spec) in array.buffers().iter().zip(&layout.buffers) {
                if let BufferSpec::FixedWidth { alignment, .. } = spec {
                    *next_offset = (*next_offset).div_ceil(*alignment) * alignment;
                }

                target[*next_offset..*next_offset + buffer.len()]
                    .copy_from_slice(buffer.as_slice());
                *next_offset += buffer.len();
            }

            for child in array.child_data() {
                fill_inner(child, next_offset, target);
            }
        }

        let mut next_offset = 0;

        fill_inner(self, &mut next_offset, target);
    }

    fn layout_with_values(&self) -> (ArrayDataLayout, Buffer) {
        fn layout_inner(
            array: &ArrayData,
            next_offset: &mut usize,
            data: &mut MutableBuffer,
        ) -> ArrayDataLayout {
            let mut buffers = Vec::new();
            let mut child_data = Vec::new();

            let layout = arrow::array::layout(array.data_type());

            for (buffer, spec) in array.buffers().iter().zip(&layout.buffers) {
                if let BufferSpec::FixedWidth { alignment, .. } = spec {
                    *next_offset = (*next_offset).div_ceil(*alignment) * alignment;
                }

                if (buffer.len() + *next_offset) - data.len() > 0 {
                    let space_needed = (buffer.len() + *next_offset) - data.len();
                    data.extend_zeros(space_needed);
                }

                data[*next_offset..*next_offset + buffer.len()].copy_from_slice(buffer.as_slice());

                buffers.push(BufferOffset {
                    len: buffer.len(),
                    offset: *next_offset,
                });

                *next_offset += buffer.len();
            }

            for child in array.child_data() {
                child_data.push(layout_inner(child, next_offset, data));
            }

            ArrayDataLayout {
                data_type: array.data_type().clone(),
                len: array.len(),
                null_bit_buffer: array.nulls().map(|b| b.validity().to_owned()),
                offset: array.offset(),
                buffers,
                child_data,
            }
        }

        let mut data = MutableBuffer::new(64);

        let mut next_offset = 0;
        let layout = layout_inner(self, &mut next_offset, &mut data);

        (layout, data.into())
    }

    fn from_layout_and_values(layout: ArrayDataLayout, values: Buffer) -> Result<ArrayData> {
        fn inner(buffer: &Buffer, layout: ArrayDataLayout) -> Result<ArrayData> {
            if buffer.is_empty() {
                return Ok(ArrayData::new_empty(&layout.data_type));
            }

            let mut buffers = Vec::new();
            let mut child_data = Vec::new();

            for BufferOffset { offset, len } in layout.buffers {
                buffers.push(buffer.slice_with_length(offset, len));
            }

            for child_data_data in layout.child_data {
                child_data.push(inner(buffer, child_data_data)?)
            }

            ArrayData::try_new(
                layout.data_type,
                layout.len,
                layout.null_bit_buffer.map(arrow::buffer::Buffer::from_vec),
                layout.offset,
                buffers,
                child_data,
            )
        }

        inner(&values, layout)
    }

    fn flattened(&self) -> Result<ArrayData> {
        let (layout, values) = self.layout_with_values();

        Self::from_layout_and_values(layout, values)
    }
}
