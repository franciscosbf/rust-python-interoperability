use chrono::{DateTime, Utc};
// TODO: Define a base class named `Discount`, with a `percentage` attribute.
//  It should be possible to access the `percentage` attribute of a `Discount`.
//  It should also be possible to modify the `percentage` attribute of a `Discount`.
//  It must be enforced that the `percentage` attribute is a float between 0. and 1.
//  Then define two subclasses:
//  - `SeasonalDiscount` that inherits from `Discount` with two additional attributes, `to` and `from_`.
//    `from_` is a datetime object that represents the start of the discount period.
//    `to` is a datetime object that represents the end of the discount period.
//     Both `from_` and `to` should be accessible and modifiable.
//     The class should enforce that `from` is before `to`.
//  - `CappedDiscount` that inherits from `Discount` with an additional attribute `cap`.
//    `cap` is a float that represents the maximum discount (in absolute value) that can be applied.
//    It should be possible to access and modify the `cap` attribute.
//    The class should enforce that `cap` is a non-zero positive float.
//
// All classes should have a method named `apply` that takes a price (float) as input and
// returns the discounted price.
// `SeasonalDiscount` should raise an `ExpiredDiscount` exception if `apply` is called but
// the current date is outside the discount period.
use pyo3::{
    create_exception,
    exceptions::{PyException, PyValueError},
    prelude::*,
    types::PyInt,
};

create_exception!(outro2, ExpiredDiscount, PyException);

#[pyclass(subclass)]
struct Discount {
    #[pyo3(get)]
    percentage: f64,
}

#[pymethods]
impl Discount {
    #[new]
    fn new(percentage: f64) -> PyResult<Self> {
        if !(0.0..=1.0).contains(&percentage) {
            return Err(PyValueError::new_err("Percentage must be between 0 and 1"));
        }

        Ok(Self { percentage })
    }

    fn apply(&self, price: u64) -> f64 {
        price as f64 - (self.percentage * price as f64)
    }

    #[setter]
    fn set_percentage(&mut self, percentage: f64) -> PyResult<()> {
        if !(0.0..=1.0).contains(&percentage) {
            return Err(PyValueError::new_err("Percentage must be between 0 and 1"));
        }

        self.percentage = percentage;

        Ok(())
    }
}

#[pyclass(extends=Discount)]
struct SeasonalDiscount {
    #[pyo3(get)]
    from_: DateTime<Utc>,
    #[pyo3(get)]
    to: DateTime<Utc>,
}

#[pymethods]
impl SeasonalDiscount {
    #[new]
    fn new(
        percentage: f64,
        from_: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> PyResult<PyClassInitializer<Self>> {
        let discount = Discount::new(percentage)?;
        if from_ >= to {
            return Err(PyValueError::new_err(
                "`from_` date must be before `to` date",
            ));
        }
        if Utc::now() > to {
            return Err(ExpiredDiscount::new_err("The discount is no longer active"));
        }
        let sdiscount = Self { from_, to };
        Ok(PyClassInitializer::from(discount).add_subclass(sdiscount))
    }

    fn apply(self_: PyRef<'_, Self>, price: u64) -> PyResult<f64> {
        if Utc::now() > self_.to {
            return Err(ExpiredDiscount::new_err("The discount is no longer active"));
        }

        Ok(self_.as_super().apply(price))
    }

    #[setter]
    fn set_from_(&mut self, from_: DateTime<Utc>) -> PyResult<()> {
        if from_ >= self.to {
            return Err(PyValueError::new_err(
                "`from_` date must be before `to` date",
            ));
        }

        self.from_ = from_;

        Ok(())
    }

    #[setter]
    fn set_to(&mut self, to: DateTime<Utc>) -> PyResult<()> {
        if self.from_ >= to {
            return Err(PyValueError::new_err(
                "`from_` date must be before `to` date",
            ));
        }

        self.to = to;

        Ok(())
    }
}

#[pyclass(extends=Discount)]
struct CappedDiscount {
    #[pyo3(get)]
    cap: f64,
}

#[pymethods]
impl CappedDiscount {
    #[new]
    fn new(percentage: f64, cap: Bound<'_, PyInt>) -> PyResult<PyClassInitializer<Self>> {
        let discount = Discount::new(percentage)?;
        let cap = cap.extract::<f64>().unwrap();
        if cap <= 0. {
            return Err(PyValueError::new_err("Cap must be a positive number"));
        }
        let sdiscount = Self { cap };
        Ok(PyClassInitializer::from(discount).add_subclass(sdiscount))
    }

    fn apply(self_: PyRef<'_, Self>, price: u64) -> f64 {
        price as f64 - (self_.as_super().percentage * price as f64).min(self_.cap)
    }

    #[setter]
    fn set_cap(&mut self, cap: Bound<'_, PyInt>) -> PyResult<()> {
        let cap = cap.extract::<f64>().unwrap();
        if cap <= 0. {
            return Err(PyValueError::new_err("Cap must be a positive number"));
        }

        self.cap = cap;

        Ok(())
    }
}

#[pymodule]
fn outro2(py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Discount>()?;
    m.add_class::<SeasonalDiscount>()?;
    m.add_class::<CappedDiscount>()?;

    m.add("ExpiredDiscount", py.get_type::<ExpiredDiscount>())?;

    Ok(())
}
