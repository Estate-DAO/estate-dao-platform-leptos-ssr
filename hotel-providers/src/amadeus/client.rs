#[derive(Clone, Debug, Default)]
pub struct AmadeusClient {
    mock: bool,
}

impl AmadeusClient {
    pub fn new_mock() -> Self {
        Self { mock: true }
    }

    pub fn is_mock(&self) -> bool {
        self.mock
    }
}
