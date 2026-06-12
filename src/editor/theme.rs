#![allow(dead_code)]

use bevy::prelude::*;

#[derive(Resource, Clone, Debug, Default)]
pub struct EditorTheme {
    pub colors: ThemeColors,
    pub sizes: ThemeSizes,
}

#[derive(Clone, Debug)]
pub struct ThemeColors {
    pub panel_bg: Color,

    pub viewport_bg: Color,

    pub bar_bg: Color,

    pub console_bg: Color,

    pub status_bg: Color,

    pub field_bg: Color,

    pub header_bg: Color,

    pub button_bg: Color,
    pub text: Color,
    pub text_dim: Color,
    pub text_faint: Color,
    pub accent: Color,
    pub selection: Color,
    pub separator: Color,
    pub splitter_idle: Color,
    pub splitter_hover: Color,
    pub error: Color,
    pub success: Color,
    pub stop: Color,
    pub build: Color,
    pub active_tab_bg: Color,
    pub tab_bar_bg: Color,
    pub tab_hover_bg: Color,
}

impl Default for ThemeColors {
    fn default() -> Self {
        Self {
            panel_bg: Color::srgb_u8(0x28, 0x28, 0x28),
            viewport_bg: Color::srgb_u8(0x15, 0x15, 0x15),
            bar_bg: Color::srgb_u8(0x15, 0x15, 0x15),
            console_bg: Color::srgb_u8(0x1a, 0x1a, 0x1a),
            status_bg: Color::srgb_u8(0x28, 0x28, 0x28),
            field_bg: Color::srgb_u8(0x1a, 0x1a, 0x1a),
            header_bg: Color::srgb_u8(0x36, 0x36, 0x36),
            button_bg: Color::srgb_u8(0x41, 0x41, 0x41),
            text: Color::srgb_u8(0xc6, 0xc6, 0xc6),
            text_dim: Color::srgb_u8(0x89, 0x89, 0x89),
            text_faint: Color::srgb_u8(0x6a, 0x6a, 0x6a),
            accent: Color::srgb_u8(0x5e, 0xa0, 0xf4),
            selection: Color::srgb_u8(0x3e, 0x3e, 0x3e),
            separator: Color::srgb_u8(0x36, 0x36, 0x36),
            splitter_idle: Color::srgb_u8(0x15, 0x15, 0x15),
            splitter_hover: Color::srgb_u8(0x36, 0x36, 0x36),
            error: Color::srgb_u8(0xff, 0x5f, 0x5f),
            success: Color::srgb_u8(0x61, 0x88, 0x45),
            stop: Color::srgb_u8(0xe1, 0x3c, 0x3c),
            build: Color::srgb_u8(0x41, 0x41, 0x41),
            active_tab_bg: Color::srgb_u8(0x28, 0x28, 0x28),
            tab_bar_bg: Color::srgb_u8(0x15, 0x15, 0x15),
            tab_hover_bg: Color::srgb_u8(0x23, 0x23, 0x23),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ThemeSizes {
    pub panel_width: f32,

    pub toolbar_height: f32,
    pub menubar_height: f32,

    pub statusbar_height: f32,
    pub tab_height: f32,

    pub inner_toolbar_height: f32,
    pub splitter_thickness: f32,
    pub heading_size: f32,

    pub row_height: f32,

    pub row_indent: f32,
    pub btn_height: f32,
    pub icon_size: f32,
    pub corner_radius: f32,
}

impl Default for ThemeSizes {
    fn default() -> Self {
        Self {
            panel_width: 260.0,
            toolbar_height: 44.0,
            menubar_height: 44.0,
            statusbar_height: 38.0,
            tab_height: 28.0,
            inner_toolbar_height: 40.0,
            splitter_thickness: 4.0,
            heading_size: 13.0,
            row_height: 21.0,
            row_indent: 16.0,
            btn_height: 28.0,
            icon_size: 16.0,
            corner_radius: 5.0,
        }
    }
}
