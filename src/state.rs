use crate::agent::{Researcher, Writer};

#[derive(Clone)]
pub struct AppState {
    pub researcher: Researcher,
    pub writer: Writer,
}

impl AppState {
    pub fn new() -> Self {
        let researcher = Researcher::new();
        let writer = Writer::new();

        Self { researcher, writer }
    }
}
