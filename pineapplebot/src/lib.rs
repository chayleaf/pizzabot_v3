use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;

#[pyclass]
struct Pizzabot(pizzabot::Pizzabot);

#[pymethods]
impl Pizzabot {
    #[new]
    fn new() -> Self {
        Self(Default::default())
    }

    fn add_message(&mut self, channel_id: &str, message: &str) {
        self.0.add_message(channel_id, message);
    }

    fn set_message(&mut self, channel_id: &str, message: &str) {
        self.0.set_message(channel_id, message);
    }

    fn load_file(&mut self, channel_id: &str, path: &str) -> PyResult<()> {
        self.0
            .load_legacy_file(channel_id, path)
            .map_err(|err| PyRuntimeError::new_err(format!("Error: {:?}", err)))
    }

    fn get_reply(&self, message: &str) -> Option<String> {
        self.0.get_reply(message)
    }

    fn repr(&self) -> String {
        format!("{:?}", self.0)
    }
}

#[pymodule]
fn pineapplebot(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Pizzabot>()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
