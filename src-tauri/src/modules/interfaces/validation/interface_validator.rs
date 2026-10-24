use crate::modules::interfaces::document::DocumentFormat;
use crate::modules::interfaces::document::InterfaceDocument;
use crate::modules::interfaces::document::InterfaceWorkspace;
use crate::modules::interfaces::document::WidgetKind;
use crate::modules::interfaces::document::WidgetNode;
use crate::modules::interfaces::evaluation::DocumentLocalizer;
use crate::modules::interfaces::validation::AdvancedValidator;
use crate::modules::interfaces::validation::WidgetVisualValidator;
use std::collections::HashSet;

#[derive(Clone)]
pub(crate) struct InterfaceValidator {
    identities: HashSet<String>,
    count: usize,
}

impl InterfaceValidator {
    pub(crate) fn new() -> Self {
        Self {
            identities: HashSet::new(),
            count: 0,
        }
    }

    pub(crate) fn validate(&mut self, workspace: &InterfaceWorkspace) -> Result<(), String> {
        self.identities.clear();
        self.count = 0;

        if workspace.format_version != DocumentFormat::VERSION || workspace.documents.len() > 100 {
            return Err("Unsupported format or too many interfaces".into());
        }

        for document in &workspace.documents {
            self.add(document)?;
        }

        Ok(())
    }

    pub(crate) fn add(&mut self, document: &InterfaceDocument) -> Result<(), String> {
        document.localization.validate()?;
        document.display.validate()?;
        self.document(document)?;

        for locale in document.localization.translations.keys() {
            let localized = DocumentLocalizer::apply(document, locale);

            Self::new().document(&localized)?;
        }

        Ok(())
    }

    fn document(&mut self, document: &InterfaceDocument) -> Result<(), String> {
        self.identity(&document.id, &document.name)?;

        if document.author.chars().count() > 128
            || document.version.chars().count() > 64
            || document.description.chars().count() > 2048
        {
            return Err("Interface information exceeds the length limit".into());
        }

        AdvancedValidator::formulas(&document.formulas)?;
        self.nodes(&document.roots, 0)?;
        Ok(())
    }

    fn identity(&mut self, id: &str, name: &str) -> Result<(), String> {
        if id.is_empty() || id.len() > 100 || name.trim().is_empty() || name.chars().count() > 128 {
            return Err("Invalid interface or element name/ID".into());
        }

        if !self.identities.insert(id.to_owned()) {
            return Err("Duplicate identifiers in document".into());
        }

        Ok(())
    }

    fn nodes(&mut self, nodes: &[WidgetNode], depth: usize) -> Result<(), String> {
        if depth > 12 {
            return Err("Group nesting is too deep".into());
        }

        for node in nodes {
            self.count += 1;

            if self.count > 8192 {
                return Err("Too many elements".into());
            }

            self.identity(&node.id, &node.name)?;
            self.layout(node)?;

            node.anchors.validate()?;

            self.appearance(node)?;
            WidgetVisualValidator::validate(node)?;
            AdvancedValidator::node(node)?;

            if !matches!(node.kind, WidgetKind::Group | WidgetKind::DynamicGroup)
                && !node.children.is_empty()
            {
                return Err("Only groups can contain children".into());
            }

            self.nodes(&node.children, depth + 1)?;
        }

        Ok(())
    }

    fn layout(&self, node: &WidgetNode) -> Result<(), String> {
        let layout = &node.layout;

        if !layout.x.is_finite()
            || !layout.y.is_finite()
            || layout.x.abs() > 100_000.0
            || layout.y.abs() > 100_000.0
            || !layout.width.is_finite()
            || !(8.0..=8192.0).contains(&layout.width)
            || !layout.height.is_finite()
            || !(8.0..=8192.0).contains(&layout.height)
        {
            return Err("Invalid position or size".into());
        }

        Ok(())
    }

    fn appearance(&self, node: &WidgetNode) -> Result<(), String> {
        let style = &node.appearance;

        for color in [&style.color, &style.background, &style.border_color] {
            if !color.starts_with('#')
                || !matches!(color.len(), 7 | 9)
                || !color[1..]
                    .chars()
                    .all(|character| character.is_ascii_hexdigit())
            {
                return Err("Color must use HEX format".into());
            }
        }

        if style.text.chars().count() > 2048
            || !style.font_size.is_finite()
            || !(6.0..=256.0).contains(&style.font_size)
            || !style.opacity.is_finite()
            || !(0.0..=1.0).contains(&style.opacity)
        {
            return Err("Invalid appearance settings".into());
        }

        if let Some(path) = &style.icon_path
            && (path.len() > 240
                || path.contains("..")
                || path.contains(':')
                || path.contains('\\'))
        {
            return Err("Invalid game icon reference".into());
        }

        Ok(())
    }
}
