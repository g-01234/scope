#![warn(clippy::all, rust_2018_idioms)]

#[macro_use]
mod macros;

pub mod shared_state;
pub use shared_state::SharedState;

mod app;
pub use app::Frontend;

pub mod abi;
pub mod backend;
pub mod providers;
pub mod utils;
pub mod wasm;
// pub use abi;

pub mod components {
    pub mod address_selector;
    pub use address_selector::AddressSelector;
    pub mod compiled_contract;
    pub use compiled_contract::CompiledContract;
    pub mod deployed_contract;
    pub use deployed_contract::DeployedContract;
    pub mod copy_button;
    pub use copy_button::CopyButton;
    pub mod test_list;
    pub use test_list::TestList;
    pub mod target_mode;
    pub use target_mode::TargetMode;
    pub mod selected_target;
    pub use selected_target::SelectedTarget;
    pub mod utility_menu;
    pub use utility_menu::UtilityMenu;
}
