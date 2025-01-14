use pyo3::{exceptions::PyValueError, prelude::*, types::PyInt};

// TODO: Add a `__new__` constructor to the `ShoppingOrder` class that takes the following arguments:
//  - `name` (non-empty string)
//  - `price` (non-zero integer)
//  - `quantity` (non-zero integer)
//  The constructor should raise a `ValueError` if any of the arguments are invalid.

#[pyclass]
struct ShoppingOrder {
    #[pyo3(get)]
    name: String,
    #[pyo3(get)]
    price: u64,
    #[pyo3(get, set)]
    quantity: u64,
}

#[pymethods]
impl ShoppingOrder {
    #[new]
    fn new(name: String, price: Bound<'_, PyInt>, quantity: Bound<'_, PyInt>) -> PyResult<Self> {
        if name.trim().is_empty() {
            return Err(PyValueError::new_err("name must be non-empty"));
        }

        let price = price
            .extract::<u64>()
            .map_err(|_| PyValueError::new_err("price must be unsigned"))?;
        if price == 0 {
            return Err(PyValueError::new_err("price must be greater than zero"));
        }

        let quantity = quantity
            .extract::<u64>()
            .map_err(|_| PyValueError::new_err("quantity must be unsigned"))?;
        if quantity == 0 {
            return Err(PyValueError::new_err("quantity must be greater than zero"));
        }

        Ok(Self {
            name,
            price,
            quantity,
        })
    }
}

#[pymodule]
fn constructors(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<ShoppingOrder>()?;
    Ok(())
}
