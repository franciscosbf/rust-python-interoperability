// TODO: Expose a function named `max_k` that takes a list of unsigned integers and return as output
//   a list containing the `k` largest numbers in the list, in descending order.
//
// Hint: you can use the `num_bigint` crate if you think it'd be useful.

use num_bigint::BigUint;
use pyo3::{
    exceptions::{PyTypeError, PyValueError},
    prelude::*,
    types::{PyInt, PyList},
};

#[pyfunction]
fn max_k(numbers: Bound<'_, PyList>, k: Bound<'_, PyInt>) -> PyResult<Vec<BigUint>> {
    let k = k
        .extract::<usize>()
        .map_err(|_| PyTypeError::new_err("k must be unsigned"))?;

    if numbers.len() < k {
        return Err(PyValueError::new_err(
            "k must be equal or less than the list size",
        ));
    }

    let mut numbers = numbers
        .iter()
        .map(|n| n.extract::<BigUint>())
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| PyTypeError::new_err("list numbers must be unsigned"))?;

    numbers.sort_by(|a, b| b.cmp(a));
    numbers.truncate(k);

    Ok(numbers)
}

#[pymodule]
fn outro1(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(max_k, m)?)?;

    Ok(())
}
