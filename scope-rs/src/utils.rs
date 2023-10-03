// STYLE UTILS
use eyre::{eyre, Result};

use egui::{FontFamily::Monospace, FontId, TextStyle};
use ethers::{
    signers::{LocalWallet, MnemonicBuilder},
    types::U256,
    utils::parse_ether,
};

use crate::shared_state;

// Queries VSCode for theme colors and uses them to set the egui theme
// Egui unfortunately currently only supports Color32, so the colors just get clamped
pub fn get_and_set_theme(
    cc: &eframe::CreationContext<'_>,
    panel_frame: &mut egui::containers::Frame,
) -> Option<()> {
    let vscode_style = crate::backend::query_for_vscode_style().unwrap_or_default();
    match vscode_style["isDarkTheme"].as_bool()? {
        // dark / high contrast
        true => catppuccin_egui::set_theme(&cc.egui_ctx, catppuccin_egui::MACCHIATO),
        // light theme
        false => catppuccin_egui::set_theme(&cc.egui_ctx, catppuccin_egui::LATTE),
    }

    let panel_rgb = hex_to_rgb(vscode_style["editor.background"].as_str().unwrap())?;
    let panel_fill = egui::Color32::from_rgb(panel_rgb.0, panel_rgb.1, panel_rgb.2);

    let menu_shadow_rgb = hex_to_rgb(vscode_style["activityBar.activeBorder"].as_str().unwrap())?;
    let menu_shadow =
        egui::Color32::from_rgb(menu_shadow_rgb.0, menu_shadow_rgb.1, menu_shadow_rgb.2);

    panel_frame.inner_margin = egui::style::Margin {
        left: 10.,
        right: 10.,
        top: 10.,
        bottom: 10.,
    };
    panel_frame.rounding = egui::Rounding {
        nw: 1.0,
        ne: 1.0,
        sw: 1.0,
        se: 1.0,
    };
    // panel_frame.shadow = eframe::epaint::Shadow {
    //     extrusion: 1.0,
    //     color: menu_shadow,
    // };
    panel_frame.fill = panel_fill;
    // panel_frame.stroke = egui::Stroke::new(2.0, Color32::GOLD);

    cc.egui_ctx.set_pixels_per_point(4.0);

    let mut style = (*cc.egui_ctx.style()).clone();
    style.text_styles = [
        (TextStyle::Heading, FontId::new(25.0, Monospace)),
        (TextStyle::Body, FontId::new(12.0, Monospace)),
        (TextStyle::Monospace, FontId::new(12.0, Monospace)),
        (TextStyle::Button, FontId::new(12.0, Monospace)),
        (TextStyle::Small, FontId::new(12.0, Monospace)),
    ]
    .into();

    cc.egui_ctx.set_style(style);

    Some(())
}

pub fn hex_to_rgb(hex: &str) -> Option<(u8, u8, u8)> {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some((r, g, b))
}

pub fn key(index: u32) -> LocalWallet {
    MnemonicBuilder::<ethers::signers::coins_bip39::English>::default()
        .phrase("stuff inherit faith park genre spread huge knee ecology private marble supreme")
        .index(index)
        .unwrap()
        .build()
        .unwrap()
}

pub fn get_max_width() -> f32 {
    let global_max_width = shared_state::STATE.max_width.read().unwrap();
    *global_max_width
}

pub fn get_max_collapsable_width() -> f32 {
    let global_max_collapsable_width = shared_state::STATE.max_collapsable_width.read().unwrap();
    *global_max_collapsable_width
}

pub fn get_radix_and_clean_str(input: &str) -> (u32, &str) {
    let mut radix = 10;
    let mut input_clean = input.clone();

    if input.starts_with("0x") {
        input_clean = input_clean.strip_prefix("0x").unwrap();
        radix = 16;
    }

    (radix, input_clean)
}

pub fn eth_str_to_u256_wei(value_string: &str) -> Result<U256> {
    match parse_ether(value_string) {
        Ok(val) => Ok(val),
        Err(_) => Err(eyre!("Error parsing msg.value {}", value_string)),
    }
}
