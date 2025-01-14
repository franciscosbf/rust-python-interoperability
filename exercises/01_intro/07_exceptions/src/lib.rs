use pyo3::{exceptions::PyTypeError, prelude::*};

#[pyfunction]
// TODO: Implement a function that returns a list containing the first `n` numbers in Fibonacci's sequence.
//  It must raise a `TypeError` if `n` is not an integer or if it is less than 0.
fn fibonacci(n: Bound<'_, PyAny>) -> PyResult<Vec<usize>> {
    let n = n
        .extract::<usize>()
        .map_err(|_| PyTypeError::new_err("invalid n"))?;

    let numbers = match n {
        0 => vec![],
        1 => vec![0],
        _ => {
            let mut numbers = vec![0, 1];
            (2..n).for_each(|n| numbers.push(numbers[n - 1] + numbers[n - 2]));

            numbers
        }
    };

    Ok(numbers)
}

#[pymodule]
fn exceptions(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(fibonacci, m)?)?;
    Ok(())
}
