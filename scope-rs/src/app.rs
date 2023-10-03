use crate::{
    backend,
    components::{ContractSelectorSection, DeployedSection, HeaderSection, TxConfigSection},
    shared_state::STATE,
    utils,
};
use egui::containers::Frame;

// Serde stuff for saving state, TODO
#[derive(serde::Deserialize, serde::Serialize, Default)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct Frontend {
    header_section: HeaderSection,
    contract_selector_section: ContractSelectorSection,
    tx_config_section: TxConfigSection,
    deploy_section: DeployedSection,

    render_configs: RenderConfigs,

    #[serde(skip)]
    panel_frame: Frame,
}

// Conditional rendering configs
#[derive(serde::Deserialize, serde::Serialize, Default)]
pub struct RenderConfigs {
    pub verbosity: i32,
    pub load_address: String,
    pub selected_name: Option<String>,
    pub show_new_address_input: bool,
    pub show_balance_input: bool,
}

impl Frontend {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        backend::initialize();
        let mut panel_frame = Frame::default();
        utils::get_and_set_theme(cc, &mut panel_frame);

        Self {
            render_configs: RenderConfigs {
                verbosity: 2,
                ..Default::default()
            },
            panel_frame,
            ..Default::default()
        }
    }
}

impl eframe::App for Frontend {
    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Main panel
        egui::CentralPanel::default()
            .frame(self.panel_frame)
            .show(ctx, |ui| {
                // Set global width (used for collapsables)
                *STATE.max_width.write().unwrap() = ui.available_width();
                *STATE.max_collapsable_width.write().unwrap() = ui.available_width() - 10.0;
                egui::warn_if_debug_build(ui);

                // Wrap the whole thing in a scroll area
                egui::ScrollArea::vertical().show(ui, |ui| {
                    self.header_section.show(ui);
                    self.contract_selector_section
                        .show(ui, &mut self.render_configs);

                    ui.separator();

                    self.tx_config_section.show(ui, &mut self.render_configs);

                    ui.separator();

                    // create collapsable headers for each address
                    self.deploy_section.show(ui, &mut self.render_configs);
                });
            });

        // Throttle refresh based on focus
        // - First case forces renders while mouse is hovering outside of extension
        //   (but focus still inside)
        // - Second case forces occasional renders when kb/mouse focus is outside extension (3sec)
        if *STATE.has_focus.read().unwrap() {
            ctx.request_repaint_after(std::time::Duration::from_millis(200));
        } else {
            ctx.request_repaint_after(std::time::Duration::from_millis(3000));
        }
    }
}
