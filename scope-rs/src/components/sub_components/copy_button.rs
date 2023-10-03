use crate::backend;
use egui::Ui;

// this should be an impl for widget or something
pub struct CopyButton {
    display_text: String,
    copy_text: String,
}
impl CopyButton {
    pub fn new(display_text: String, copy_text: impl Into<String>) -> Self {
        Self {
            display_text,
            copy_text: copy_text.into(),
        }
    }

    pub fn show(&mut self, ui: &mut Ui) {
        if ui.button(self.display_text.clone()).clicked() {
            ui.output_mut(|o| o.copied_text = self.copy_text.clone());
        }
    }
}
