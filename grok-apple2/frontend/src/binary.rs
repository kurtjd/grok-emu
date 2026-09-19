use eframe::egui;

#[derive(Clone, Copy, PartialEq, Eq)]
enum BinaryFormat {
    Dos33,
    Manual,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum AddressBase {
    Decimal,
    Hexadecimal,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinaryLoadAction {
    LoadDos33,
    LoadDirect { address: usize },
    Cancel,
}

pub struct BinaryLoadDialog {
    name: String,
    pub data: Vec<u8>,
    format: BinaryFormat,
    base: AddressBase,
    address: String,
    pub error: Option<String>,
    focus_address: bool,
}

impl BinaryLoadDialog {
    pub fn new(name: String, data: Vec<u8>) -> Self {
        Self {
            name,
            data,
            format: BinaryFormat::Dos33,
            base: AddressBase::Hexadecimal,
            address: "0800".to_owned(),
            error: None,
            focus_address: true,
        }
    }

    pub fn show(&mut self, ctx: &egui::Context) -> Option<BinaryLoadAction> {
        let response = egui::Modal::new(egui::Id::new("load_binary")).show(ctx, |ui| {
            ui.set_width(320.0_f32.min((ctx.content_rect().width() - 32.0).max(0.0)));
            ui.heading("Load Binary");
            ui.add(egui::Label::new(&self.name).wrap());
            let data_size = match self.format {
                BinaryFormat::Dos33 => self.data.len().saturating_sub(4),
                BinaryFormat::Manual => self.data.len(),
            };
            ui.weak(format!("{data_size} bytes"));
            ui.separator();

            let previous_format = self.format;
            ui.horizontal_wrapped(|ui| {
                ui.selectable_value(&mut self.format, BinaryFormat::Dos33, "DOS 3.3 header");
                ui.selectable_value(&mut self.format, BinaryFormat::Manual, "Manual address");
            });
            if self.format != previous_format {
                self.error = None;
                self.focus_address = true;
            }

            let mut submitted = false;
            let address = match self.format {
                BinaryFormat::Dos33 => {
                    ui.label("Load address");
                    self.data
                        .get(..4)
                        .map(|header| usize::from(u16::from_le_bytes([header[0], header[1]])))
                }
                BinaryFormat::Manual => {
                    let mut base = self.base;
                    ui.horizontal_wrapped(|ui| {
                        ui.label("Load address");
                        ui.selectable_value(&mut base, AddressBase::Decimal, "Decimal");
                        ui.selectable_value(&mut base, AddressBase::Hexadecimal, "Hex");
                    });
                    self.set_base(base);

                    let input = ui.add(
                        egui::TextEdit::singleline(&mut self.address)
                            .id(egui::Id::new("binary_load_address"))
                            .font(egui::TextStyle::Monospace)
                            .desired_width(f32::INFINITY),
                    );
                    if self.focus_address {
                        input.request_focus();
                        self.focus_address = false;
                    }
                    if input.changed() {
                        self.error = None;
                    }
                    submitted =
                        input.lost_focus() && ui.input(|input| input.key_pressed(egui::Key::Enter));
                    self.address().ok()
                }
            };
            if let Some(address) = address {
                ui.horizontal_wrapped(|ui| {
                    ui.monospace(format!("Decimal: {address}"));
                    ui.monospace(format!("Hex: ${address:04X}"));
                });
            }

            let load_action = self.load_action();
            if let Some(error) = load_action.err().or(self.error.as_deref()) {
                ui.add(
                    egui::Label::new(egui::RichText::new(error).color(ui.visuals().error_fg_color))
                        .wrap(),
                );
            }

            ui.separator();
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let load = ui
                    .add_enabled(load_action.is_ok(), egui::Button::new("Load"))
                    .clicked();
                if ui.button("Cancel").clicked() {
                    return Some(BinaryLoadAction::Cancel);
                }
                if load || submitted {
                    return load_action.ok();
                }
                None
            })
            .inner
        });
        if response.should_close() {
            Some(BinaryLoadAction::Cancel)
        } else {
            response.inner
        }
    }

    fn load_action(&self) -> Result<BinaryLoadAction, &'static str> {
        if self.data.is_empty() {
            return Err("The binary file is empty.");
        }
        match self.format {
            BinaryFormat::Dos33 if self.data.len() < 4 => {
                Err("The DOS 3.3 header is incomplete (4 bytes required).")
            }
            BinaryFormat::Dos33 if self.data.len() == 4 => {
                Err("The binary has no data after its DOS 3.3 header.")
            }
            BinaryFormat::Dos33 => Ok(BinaryLoadAction::LoadDos33),
            BinaryFormat::Manual => self
                .address()
                .map(|address| BinaryLoadAction::LoadDirect { address }),
        }
    }

    fn address(&self) -> Result<usize, &'static str> {
        let text = self.address.trim();
        let (digits, radix, error) = match self.base {
            AddressBase::Decimal => (text, 10, "Invalid decimal address (0 to 65535)."),
            AddressBase::Hexadecimal => (
                text.strip_prefix('$')
                    .or_else(|| text.strip_prefix("0x"))
                    .or_else(|| text.strip_prefix("0X"))
                    .unwrap_or(text),
                16,
                "Invalid hexadecimal address ($0000 to $FFFF).",
            ),
        };
        u16::from_str_radix(digits, radix)
            .map(usize::from)
            .map_err(|_| error)
    }

    fn set_base(&mut self, base: AddressBase) {
        if self.base == base {
            return;
        }
        if let Ok(address) = self.address() {
            self.address = match base {
                AddressBase::Decimal => address.to_string(),
                AddressBase::Hexadecimal => format!("{address:04X}"),
            };
        }
        self.base = base;
        self.error = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render_dialog(
        ctx: &egui::Context,
        dialog: &mut BinaryLoadDialog,
        events: Vec<egui::Event>,
    ) -> (Option<BinaryLoadAction>, egui::FullOutput) {
        let mut action = None;
        let output = ctx.run_ui(
            egui::RawInput {
                events,
                ..Default::default()
            },
            |ui| {
                if let Some(request) = dialog.show(ui.ctx()) {
                    action = Some(request);
                }
            },
        );
        (action, output)
    }

    fn click_dialog_button(
        ctx: &egui::Context,
        dialog: &mut BinaryLoadDialog,
        label: &str,
    ) -> Option<BinaryLoadAction> {
        let _ = render_dialog(ctx, dialog, Vec::new());
        let (_, output) = render_dialog(ctx, dialog, Vec::new());
        let position = output
            .shapes
            .iter()
            .find_map(|shape| match &shape.shape {
                egui::Shape::Text(text) if text.galley.text() == label => {
                    Some(text.pos + text.galley.rect.center().to_vec2())
                }
                _ => None,
            })
            .unwrap_or_else(|| panic!("Missing dialog button: {label}"));
        render_dialog(
            ctx,
            dialog,
            vec![
                egui::Event::PointerMoved(position),
                egui::Event::PointerButton {
                    pos: position,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: egui::Modifiers::NONE,
                },
                egui::Event::PointerButton {
                    pos: position,
                    button: egui::PointerButton::Primary,
                    pressed: false,
                    modifiers: egui::Modifiers::NONE,
                },
            ],
        )
        .0
    }

    #[test]
    fn binary_dialog_byte_count_excludes_only_dos33_header() {
        let ctx = egui::Context::default();
        let mut dialog = BinaryLoadDialog::new("test.bin".to_owned(), vec![0; 42]);

        for (format, file_size, expected) in [
            (BinaryFormat::Dos33, 42, "38 bytes"),
            (BinaryFormat::Manual, 42, "42 bytes"),
            (BinaryFormat::Dos33, 42, "38 bytes"),
            (BinaryFormat::Dos33, 4, "0 bytes"),
            (BinaryFormat::Dos33, 2, "0 bytes"),
            (BinaryFormat::Manual, 2, "2 bytes"),
            (BinaryFormat::Dos33, 0, "0 bytes"),
        ] {
            dialog.format = format;
            dialog.data.resize(file_size, 0);
            let _ = render_dialog(&ctx, &mut dialog, Vec::new());
            let (_, output) = render_dialog(&ctx, &mut dialog, Vec::new());
            assert!(output.shapes.iter().any(|shape| {
                matches!(&shape.shape, egui::Shape::Text(text) if text.galley.text() == expected)
            }));
        }
    }

    #[test]
    fn binary_dialog_buttons_switch_format_and_base() {
        let ctx = egui::Context::default();
        let mut dialog =
            BinaryLoadDialog::new("test.bin".to_owned(), vec![0x00, 0x20, 0x01, 0x00, 0x60]);

        assert_eq!(
            click_dialog_button(&ctx, &mut dialog, "Load"),
            Some(BinaryLoadAction::LoadDos33)
        );
        assert_eq!(
            click_dialog_button(&ctx, &mut dialog, "Manual address"),
            None
        );
        assert!(dialog.format == BinaryFormat::Manual);
        assert_eq!(click_dialog_button(&ctx, &mut dialog, "Decimal"), None);
        assert_eq!(dialog.address, "2048");
        assert_eq!(click_dialog_button(&ctx, &mut dialog, "Hex"), None);
        assert_eq!(dialog.address, "0800");
        assert_eq!(
            click_dialog_button(&ctx, &mut dialog, "Load"),
            Some(BinaryLoadAction::LoadDirect { address: 2048 })
        );
        assert_eq!(
            click_dialog_button(&ctx, &mut dialog, "DOS 3.3 header"),
            None
        );
        assert!(dialog.format == BinaryFormat::Dos33);
        assert_eq!(
            click_dialog_button(&ctx, &mut dialog, "Cancel"),
            Some(BinaryLoadAction::Cancel)
        );
    }

    #[test]
    fn binary_dialog_disables_loading_incomplete_header() {
        let ctx = egui::Context::default();
        let mut dialog = BinaryLoadDialog::new("short.bin".to_owned(), vec![0x00, 0x08]);

        assert_eq!(click_dialog_button(&ctx, &mut dialog, "Load"), None);
        assert_eq!(
            click_dialog_button(&ctx, &mut dialog, "Manual address"),
            None
        );
        assert_eq!(
            click_dialog_button(&ctx, &mut dialog, "Load"),
            Some(BinaryLoadAction::LoadDirect { address: 2048 })
        );
    }

    #[test]
    fn binary_defaults_to_dos33_without_using_manual_address() {
        let mut dialog =
            BinaryLoadDialog::new("test.bin".to_owned(), vec![0x00, 0x08, 0x01, 0x00, 0x60]);
        dialog.address = "not an address".to_owned();
        assert_eq!(dialog.load_action(), Ok(BinaryLoadAction::LoadDos33));
    }

    #[test]
    fn binary_manual_mode_preserves_address_and_all_file_bytes() {
        let data = vec![0x00, 0x08, 0x01, 0x00, 0x60];
        let mut dialog = BinaryLoadDialog::new("test.bin".to_owned(), data.clone());
        dialog.format = BinaryFormat::Manual;
        dialog.address = "0300".to_owned();
        dialog.set_base(AddressBase::Decimal);
        assert_eq!(dialog.address, "768");
        assert_eq!(
            dialog.load_action(),
            Ok(BinaryLoadAction::LoadDirect { address: 768 })
        );
        dialog.format = BinaryFormat::Dos33;
        assert_eq!(dialog.load_action(), Ok(BinaryLoadAction::LoadDos33));
        dialog.format = BinaryFormat::Manual;
        assert_eq!(
            dialog.load_action(),
            Ok(BinaryLoadAction::LoadDirect { address: 768 })
        );
        assert_eq!(dialog.data, data);
    }

    #[test]
    fn binary_rejects_empty_files_and_incomplete_dos33_files() {
        let binary = [0x00, 0x08, 0x01, 0x00, 0x60];
        for length in 0..binary.len() {
            let mut dialog =
                BinaryLoadDialog::new("test.bin".to_owned(), binary[..length].to_vec());
            assert!(dialog.load_action().is_err());
            dialog.format = BinaryFormat::Manual;
            assert_eq!(dialog.load_action().is_ok(), length > 0);
        }
    }

    #[test]
    fn binary_address_accepts_decimal_and_hexadecimal() {
        let mut dialog = BinaryLoadDialog::new("test.bin".to_owned(), vec![0x60]);
        for text in ["0800", "$0800", "0x0800", "0X0800", " 800 "] {
            dialog.address = text.to_owned();
            assert_eq!(dialog.address(), Ok(2048), "{text}");
        }
        dialog.set_base(AddressBase::Decimal);
        dialog.address = " 2048 ".to_owned();
        assert_eq!(dialog.address(), Ok(2048));
        dialog.set_base(AddressBase::Hexadecimal);
        dialog.address = "bfff".to_owned();
        assert_eq!(dialog.address(), Ok(0xBFFF));
    }

    #[test]
    fn binary_address_switching_base_preserves_value() {
        let mut dialog = BinaryLoadDialog::new("test.bin".to_owned(), vec![0x60]);
        dialog.set_base(AddressBase::Decimal);
        assert_eq!(dialog.address, "2048");
        assert_eq!(dialog.address(), Ok(0x0800));
        dialog.address = "49151".to_owned();
        dialog.set_base(AddressBase::Hexadecimal);
        assert_eq!(dialog.address, "BFFF");
        assert_eq!(dialog.address(), Ok(49151));
    }

    #[test]
    fn binary_address_rejects_invalid_input_without_clamping() {
        let mut dialog = BinaryLoadDialog::new("test.bin".to_owned(), vec![0x60]);
        for text in ["", " ", "$", "0x", "-1", "xyz", "10000"] {
            dialog.address = text.to_owned();
            assert!(dialog.address().is_err(), "{text}");
        }
        dialog.set_base(AddressBase::Decimal);
        for text in ["", "-1", "ABCD", "$0800", "0x0800", "65536", "1.5"] {
            dialog.address = text.to_owned();
            assert!(dialog.address().is_err(), "{text}");
        }
    }
}
