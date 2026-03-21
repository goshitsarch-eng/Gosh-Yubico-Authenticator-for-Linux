use iced::color;
use iced::Color;

/// Application color palette
pub struct Palette;

impl Palette {
    // Brand colors
    pub const PRIMARY: Color = color!(0x3584e4);
    pub const PRIMARY_DARK: Color = color!(0x1c71d8);
    pub const ACCENT: Color = color!(0xf6d32d);

    // Surface colors - light theme
    pub const SURFACE_LIGHT: Color = color!(0xfafafa);
    pub const CARD_LIGHT: Color = Color::WHITE;
    pub const TEXT_LIGHT: Color = color!(0x1e1e1e);
    pub const TEXT_SECONDARY_LIGHT: Color = color!(0x6e6e6e);
    pub const BORDER_LIGHT: Color = color!(0xe0e0e0);

    // Surface colors - dark theme
    pub const SURFACE_DARK: Color = color!(0x1e1e1e);
    pub const CARD_DARK: Color = color!(0x2d2d2d);
    pub const TEXT_DARK: Color = color!(0xf0f0f0);
    pub const TEXT_SECONDARY_DARK: Color = color!(0xa0a0a0);
    pub const BORDER_DARK: Color = color!(0x404040);

    // Status colors
    pub const SUCCESS: Color = color!(0x33d17a);
    pub const ERROR: Color = color!(0xe01b24);
    pub const WARNING: Color = color!(0xf5c211);

    // Countdown colors
    pub const COUNTDOWN_SAFE: Color = color!(0x33d17a);
    pub const COUNTDOWN_WARNING: Color = color!(0xf5c211);
    pub const COUNTDOWN_DANGER: Color = color!(0xe01b24);
}

/// Get countdown color based on remaining seconds
pub fn countdown_color(remaining: u32, period: u32) -> Color {
    let ratio = remaining as f32 / period as f32;
    if ratio > 0.5 {
        Palette::COUNTDOWN_SAFE
    } else if ratio > 0.2 {
        Palette::COUNTDOWN_WARNING
    } else {
        Palette::COUNTDOWN_DANGER
    }
}
