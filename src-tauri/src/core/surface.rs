#[derive(Clone, Copy)]
pub(crate) enum Surface {
    Editor,
}

impl Surface {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Editor => "editor",
        }
    }

    pub(crate) fn require(self, label: &str) -> Result<(), String> {
        if label == self.label() {
            return Ok(());
        }

        Err("Editor access required".into())
    }
}
