pub(crate) struct DocumentFormat;

impl DocumentFormat {
    pub(crate) const VERSION: u32 = 1;
    pub(crate) const PREFIX: &str = "!AURION:1!";

    pub(crate) fn unsupported(error: &serde_json::Error) -> bool {
        let text = error.to_string();

        text.starts_with("unknown field") || text.starts_with("unknown variant")
    }
}
