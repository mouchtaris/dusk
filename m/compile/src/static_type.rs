#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Type {}

impl Type {
    pub fn any() -> Self {
        Self {}
    }

    pub fn process_handle() -> Self {
        Self {}
    }
}
