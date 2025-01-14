use std::collections::HashMap;

use pyo3::prelude::*;

fn _fibonacci(n: u64, computed: &mut HashMap<u64, u64>) -> u64 {
    match n {
        0 | 1 => n,
        _ => computed.get(&n).cloned().unwrap_or_else(|| {
            let _n = _fibonacci(n - 1, computed) + _fibonacci(n - 2, computed);
            computed.insert(n, _n);
            _n
        }),
    }
}

#[pyfunction]
// TODO: Implement a function that returns a list containing the first `n` numbers in Fibonacci's sequence.
fn fibonacci(n: u64) -> Vec<u64> {
    let mut computed = HashMap::new();

    (0..n).map(|n| _fibonacci(n, &mut computed)).collect()
}

#[pymodule]
fn output(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(fibonacci, m)?)?;
    Ok(())
}
