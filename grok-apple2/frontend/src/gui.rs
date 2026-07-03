use eframe::egui;

/// An action requested via the toolbar, executed by the app.
pub enum UiAction {
    Reset,
    Quit,
}

/// Render the top toolbar. Returns the action the user triggered, if any.
///
/// This is intentionally free of any emulator or window state so the toolbar
/// stays a pure view: it only reports *what was clicked*, never acts on it.
pub fn menu_bar(ui: &mut egui::Ui) -> Option<UiAction> {
    let mut action = None;

    egui::containers::menu::MenuBar::new().ui(ui, |ui| {
        ui.menu_button("File", |ui| {
            if ui
                .add(egui::Button::new("Reset").shortcut_text("F2"))
                .clicked()
            {
                action = Some(UiAction::Reset);
                ui.close();
            }
            ui.separator();
            if ui.button("Quit").clicked() {
                action = Some(UiAction::Quit);
                ui.close();
            }
        });
    });

    action
}
