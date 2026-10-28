#[derive(Clone, Copy)]
pub(super) enum TrayAction {
    Editor,
    Quit,
}

impl TrayAction {
    pub(super) const ALL: [Self; 2] = [Self::Editor, Self::Quit];

    pub(super) fn id(self) -> &'static str {
        match self {
            Self::Editor => "editor",
            Self::Quit => "quit",
        }
    }

    pub(super) fn parse(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|action| action.id() == id)
    }
}
