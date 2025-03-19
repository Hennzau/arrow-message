use arrow::{array::ArrayData, pyarrow::PyArrowType};
use pyo3::prelude::*;

#[pyfunction]
fn sum_as_string(a: usize, b: usize) -> PyResult<String> {
    Ok((a + b).to_string())
}

#[pyfunction]
fn help_function(array: PyArrowType<ArrayData>) -> PyResult<()> {
    let array = array.0;

    println!("Array: {:?}", array);
    Ok(())
}

#[pymodule]
fn pyarrow_message(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(sum_as_string, m)?)?;
    m.add_function(wrap_pyfunction!(help_function, m)?)?;
    Ok(())
}
