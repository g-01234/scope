use crate::{
    backend,
    components::{
        address_selector::AddressSelector, CopyButton, SelectedTarget, TargetMode, UtilityMenu,
    },
    shared_state::STATE,
    utils,
};
use egui::containers::Frame;

// Serde stuff for saving state, TODO
#[derive(serde::Deserialize, serde::Serialize, Default)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct Frontend {
    // Example stuff:
    target: Option<SelectedTarget>,

    address_selector: AddressSelector,

    #[serde(skip)]
    panel_frame: Frame,

    render_configs: RenderConfigs,

    shutdown: bool,
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
    /// Called by the frame work to save state before shutdown.

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // let old = ctx.style().visuals.clone();
        // ctx.set_visuals(egui::Visuals { ..old });
        // ctx.style()

        // Main panel
        egui::CentralPanel::default()
            .frame(self.panel_frame)
            .show(ctx, |ui| {
                // Set global width
                *STATE.max_width.write().unwrap() = ui.available_width();
                *STATE.max_collapsable_width.write().unwrap() = ui.available_width() - 10.0;
                egui::warn_if_debug_build(ui);

                // Wrap it in a scroll area
                egui::ScrollArea::vertical().show(ui, |ui| {
                    self.render_header(ui);

                    self.render_contract_selector(ui);
                    self.render_selected_target(ui);

                    ui.separator();

                    ui.label("Transaction Configs");
                    self.render_sender_selector(ui);
                    self.render_value_input(ui);

                    ui.separator();

                    // create collapsable headers for each address
                    self.render_deployed_contracts(ui);
                });
            });

        // Throttle refresh based on focus
        // - First case forces renders while mouse is hovering outside of extension
        //   (but focus still inside)
        // - Second case forces occasional renders when kb/mouse focus is outside extension
        if *STATE.has_focus.read().unwrap() {
            ctx.request_repaint_after(std::time::Duration::from_millis(200));
        } else {
            ctx.request_repaint_after(std::time::Duration::from_millis(2000));
        }
    }

    fn as_any_mut(&mut self) -> Option<&mut dyn std::any::Any> {
        None
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        //     eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {}

    fn auto_save_interval(&self) -> std::time::Duration {
        std::time::Duration::from_secs(30)
    }

    fn max_size_points(&self) -> egui::Vec2 {
        egui::Vec2::INFINITY
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        // NOTE: a bright gray makes the shadows of the windows look weird.
        // We use a bit of transparency so that if the user switches on the
        // `transparent()` option they get immediate results.
        egui::Color32::from_rgba_unmultiplied(12, 12, 12, 180).to_normalized_gamma_f32()

        // _visuals.window_fill() would also be a natural choice
    }

    fn persist_native_window(&self) -> bool {
        true
    }

    fn persist_egui_memory(&self) -> bool {
        true
    }

    fn warm_up_enabled(&self) -> bool {
        false
    }

    fn post_rendering(&mut self, _window_size_px: [u32; 2], _frame: &eframe::Frame) {}
}

impl Frontend {
    fn render_header(&mut self, ui: &mut egui::Ui) {
        // Top horizontal (title and root utility menu)
        ui.horizontal(|ui| {
            // ui.heading(RichText::new("scope 🔭").font(FontId::monospace(14.0)));
            ui.horizontal(|ui| {
                ui.heading("scope🔭");
                ui.label("(version: α)");
            });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                UtilityMenu::show_for_root(ui);
            });
        });

        // Refresh and compile buttons
        ui.horizontal(|ui| {
            if ui.button("🔄").clicked() {
                backend::query_for_open_files();
            }
            if ui.button("Compile").clicked() {
                backend::send_forge_build();
            }
        });
        ui.separator();
    }

    fn render_contract_selector(&mut self, ui: &mut egui::Ui) {
        let selected_name = &mut self.render_configs.selected_name;

        // Initialize a Vec to store tuples of the formatted name and filepath
        let mut contract_list: Vec<(String, String)> = Vec::new();

        for file_path in &(*STATE.open_files.read().unwrap()) {
            if !file_path.is_empty() {
                let split_path: Vec<&str> = file_path.split('/').collect();
                let contract_name = split_path
                    .last()
                    .unwrap()
                    .trim_end_matches(".json")
                    .to_string();
                let file_name = split_path[split_path.len() - 2].to_string();
                let formatted_name = format!("{}:{}", file_name, contract_name);
                contract_list.push((formatted_name, file_path.clone()));
            }
        }

        // Add special cases to the contract_list
        contract_list.push((
            "Deploy raw bytecode".to_string(),
            "DeployRawBytecode".to_string(),
        ));
        contract_list.push((
            "Load address without ABI".to_string(),
            "LoadWithoutABI".to_string(),
        ));

        let prev_selected = selected_name.clone();

        egui::ComboBox::from_id_source("contract_selector")
            .selected_text(
                selected_name
                    .clone()
                    .unwrap_or_else(|| "Select Contract".to_string()),
            )
            .width(ui.available_width())
            .show_ui(ui, |ui| {
                for (contract_name, _) in &contract_list {
                    ui.selectable_value(selected_name, Some(contract_name.clone()), contract_name);
                }
            });

        if prev_selected != *selected_name {
            if let Some(ref new_selection) = *selected_name {
                if let Some((formatted_name, file_path)) =
                    contract_list.iter().find(|(name, _)| name == new_selection)
                {
                    match file_path.as_str() {
                        "DeployRawBytecode" => {
                            self.target = Some(SelectedTarget::new_deploy_raw(
                                "Arbitrary/Bytecode".to_string(),
                                String::new(),
                            ));
                        }
                        "LoadWithoutABI" => {
                            self.target =
                                Some(SelectedTarget::new_load_raw("Load/Load".to_string()));
                        }
                        file_path => {
                            if formatted_name.contains(".t.sol") {
                                self.target =
                                    Some(SelectedTarget::new_foundry_test(file_path.to_string()));
                            } else {
                                self.target =
                                    Some(SelectedTarget::new_compiled(file_path.to_string()));
                            }
                            backend::query_for_compiled_solidity(file_path.to_string());
                        }
                    }
                }
            }
        }
    }

    fn render_selected_target(&mut self, ui: &mut egui::Ui) {
        if let Some(target) = &mut self.target {
            // If we've received compiled solidity from VSCode, consume it from shared state
            // as the contract for target.contract and set to None in shared state.

            if let Some(received_compiled) = STATE.target_compiled.write().unwrap().take() {
                match &mut target.mode {
                    TargetMode::Compiled { contract, .. } => {
                        *contract = Some(received_compiled.clone());
                    }
                    TargetMode::FoundryTest { contract, .. } => {
                        *contract = Some(received_compiled.clone());
                    }
                    // Other cases
                    _ => {}
                }
            }

            target.show(ui, &mut self.render_configs);

            // Handle compile occurrence
            let compile_occurred = *STATE.completed_compile.read().unwrap();
            if compile_occurred == Some(true) {
                if let TargetMode::Compiled { file_path, .. } = &target.mode {
                    backend::query_for_compiled_solidity(file_path.to_string());
                }
                backend::query_for_open_files();
                *STATE.completed_compile.write().unwrap() = Some(false);
            }
        }
    }

    // Renders the "from" dropdown and updates shared state if a new sender is selected
    fn render_sender_selector(&mut self, ui: &mut egui::Ui) {
        self.address_selector.show(ui, &mut self.render_configs);
    }

    // Renders the value input and updates shared state if a new value is entered
    fn render_value_input(&mut self, ui: &mut egui::Ui) {
        // Get the tx_configs write lock
        let mut tx_configs = STATE.tx_configs.write().unwrap();

        ui.horizontal(|ui| {
            ui.label("Value: ");
            ui.add(
                egui::TextEdit::singleline(&mut tx_configs.value)
                    .hint_text("0 (ether)")
                    .desired_width(ui.available_width() * 0.5),
            );
        });
    }

    fn render_deployed_contracts(&mut self, ui: &mut egui::Ui) {
        let deployed_contracts = &mut (*STATE.deployed_contracts.write().unwrap());

        // sort deployed_contracts by deployed block number; need to do this every time unfortunately
        // as deployed_contracts.remove() will break ordering
        deployed_contracts
            .sort_by(|_, a, _, b| a.deployed_block.number.cmp(&b.deployed_block.number));
        // create collapsable headers for each address
        // can we avoid cloning contracts_map here
        for address in deployed_contracts //.sort_by(|k, v| v.)
            .keys()
            .cloned()
            .collect::<Vec<String>>()
        {
            let name = deployed_contracts.get(&address).unwrap().name.clone();
            ui.horizontal(|ui| {
                let mut should_remove = false;

                if let Some(deployed) = deployed_contracts.get_mut(&address) {
                    let avail_width = ui.available_width();
                    ui.set_max_width(utils::get_max_collapsable_width());
                    let collapsing_resp =
                        ui.collapsing(format!("{:?} - {:?}", name, address.clone()), |ui| {
                            deployed.show(ui);
                        });

                    // let add_space = collapsing_resp.fully_open();
                    let add_space = collapsing_resp.openness;
                    // ui.set_max_width(prev_width);
                    // ui.add_space(prev_width * 0.75);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                        // don't love, bounces for a sec
                        // if add_space {
                        //     ui.add_space(avail_width * 0.025);
                        // }
                        ui.add_space(add_space * 7.5);
                        // ui.add_space(ui.available_width() * 0.5);
                        if ui.button("❌").clicked() {
                            should_remove = true;
                        }

                        UtilityMenu::show_for_deployed(ui, deployed);

                        CopyButton::new("📋".to_string(), deployed.address_string.clone()).show(ui);
                    });
                }
                // borrow checker fun
                if should_remove {
                    deployed_contracts.remove(&address);
                }
            });

            ui.separator();
        }
    }
}

// impl Frontend {
//     fn add_select_contract_menu(&mut self, ui: &mut egui::Ui) {
//         ui.menu_button("Select Contract ", |ui| {
//             for file_path in &(*STATE.open_files.read().unwrap()) {
//                 // handle this better; maybe in recieve_open_Files()
//                 if !file_path.is_empty() {
//                     let split_path: Vec<&str> = file_path.split('/').collect();
//                     let contract_name = split_path
//                         .last()
//                         .unwrap()
//                         .trim_end_matches(".json")
//                         .to_string();
//                     let file_name = split_path[split_path.len() - 2].to_string();
//                     // let filename = file_path.split('/').last().unwrap();
//                     if ui
//                         .button(format!("{}:{}", file_name, contract_name))
//                         .clicked()
//                     {
//                         self.target = Some(SelectedTarget::new(file_path.clone()));

//                         backend::query_for_compiled_solidity(
//                             file_path.to_string(),
//                             ReceivedFileDestination::Test,
//                         );
//                         ui.close_menu();
//                     }
//                     ui.separator();
//                 }
//             }

//             // handle arbitrary bytecode case
//             if ui.button("Deploy raw bytecode".to_string()).clicked() {
//                 self.target = Some(SelectedTarget::new("Arbitrary/Bytecode".to_string()));
//                 self.target.as_mut().unwrap().is_raw_bytecode = true;
//                 ui.close_menu();
//             }
//             // handle arbitrary bytecode case
//             if ui.button("Load address without ABI".to_string()).clicked() {
//                 self.target = Some(SelectedTarget::new("Load/Load".to_string()));
//                 self.target.as_mut().unwrap().is_empty_load = true;
//                 ui.close_menu();
//             }
//         });
//     }

//     fn handle_selected_target(&mut self, ui: &mut egui::Ui, target: &SelectedTarget) {
//         // if let Some(contract) = &mut target.contract {
//         //     UtilityMenu::show_for_selected(ui, contract)
//         // }

//         // if user has clicked compile, need to refresh our target's compiled artifact
//         // kinda ugly but need to release lock to write to it
//         let compile_occurred = *STATE.completed_compile.read().unwrap();
//         if compile_occurred == Some(true) {
//             *STATE.completed_compile.write().unwrap() = Some(false);
//             backend::query_for_compiled_solidity(
//                 target.file_path.to_string(),
//                 ReceivedFileDestination::Test,
//             );
//             backend::query_for_open_files();
//         }
//         // Show the selected target

//         // Handle case where the target is a test file - want to present tests we can run
//         if let Some(compiled) = &target.contract {
//             // if let Some(target.compiled_contract)
//             if compiled.file_name.ends_with(".t.sol") {
//                 ui.label(format!("Target: {:?}", compiled.contract_name));

//                 let mut collapsable = TestsCollapsable::default();
//                 collapsable.name = target.contract_name.clone();
//                 collapsable.show(ui, compiled.clone(), &mut self.verbosity);
//                 ui.separator();
//             }
//         }
//         // Load address
//         ui.horizontal(|ui| {
//             let address: &mut String = &mut self.address_to_load;
//             // can i not just use ui.singleline or something
//             egui::TextEdit::singleline(address)
//                 .hint_text("Address to load".to_string())
//                 // .desired_width(f32::INFINITY)
//                 .show(ui);
//             if ui.button("Load at").clicked() {
//                 backend::load_at_address_wrapper(None, address.to_string())
//             }
//         });
//     }
// }

// use egui::Ui;

// #[derive(Default, Clone)]
// pub struct DummyContract {
//     storage_slot_input: String,
// }

// impl DummyContract {
//     pub fn new() -> Self {
//         Self {
//             storage_slot_input: String::new(),
//         }
//     }

//     pub fn show(&mut self, ui: &mut Ui) {
//         self.show_storage(ui);
//     }

//     fn show_storage(&mut self, ui: &mut Ui) {
//         ui.horizontal_top(|ui| {
//             if ui.button("Storage").clicked() {
//                 // Do something
//             }
//             ui.horizontal(|ui| {
//                 let text: &mut String = &mut self.storage_slot_input;
//                 egui::TextEdit::singleline(text)
//                     .hint_text("slot")
//                     .desired_width(utils::get_max_collapsable_width())
//                     .show(ui);
//                 // ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
//                 if ui.button("❌").clicked() {}
//                 // })
//             });
//         });
//     }
// }

// pub fn show_collapsables(ui: &mut Ui) {
//     // ui.set_max_width(ui.available_width()); // Set a maximum width for the container
//     // let prev_available_width = ui.available_width();

//     let mut deployed_contracts = vec![
//         ("0xAddress1".to_string(), DummyContract::new()),
//         ("0xAddress2".to_string(), DummyContract::new()),
//         ("0xAddress3".to_string(), DummyContract::new()),
//     ];

//     for i in 0..deployed_contracts.len() {
//         ui.horizontal(|ui| {
//             let avail_width = ui.available_width();
//             println!("Available width: {}", avail_width); // Or use logging based on your setup

//             let (address, contract) = &mut deployed_contracts[i];
//             let collapsing = egui::CollapsingHeader::new(format!("Contract - {}", address))
//                 .id_source(format!("Contract - {}", address))
//                 .default_open(false);

//             collapsing.show(ui, |ui| {
//                 ui.set_max_width(utils::get_max_collapsable_width()); // Also set the maximum width for the inner UI
//                 contract.show(ui);
//             });
//         });

//         ui.separator();
//     }
// }
