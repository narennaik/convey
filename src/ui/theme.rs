use iced::Color;

/// Solarized Light Theme - Warm, easy on the eyes
/// Cream background with muted, harmonious colors
pub struct WillowDark;

impl WillowDark {
    // Core colors - Solarized Light base
    pub const BACKGROUND: Color = Color::from_rgb(
        0xfd as f32 / 255.0,
        0xf6 as f32 / 255.0,
        0xe3 as f32 / 255.0,
    ); // #fdf6e3 - cream background

    pub const SURFACE: Color = Color::from_rgb(
        0xee as f32 / 255.0,
        0xe8 as f32 / 255.0,
        0xd5 as f32 / 255.0,
    ); // #eee8d5 - light beige for panels

    pub const SURFACE_BORDER: Color = Color::from_rgb(
        0x93 as f32 / 255.0,
        0xa1 as f32 / 255.0,
        0xa1 as f32 / 255.0,
    ); // #93a1a1 - muted gray border

    pub const SURFACE_HOVER: Color = Color::from_rgb(
        0xe8 as f32 / 255.0,
        0xe2 as f32 / 255.0,
        0xcf as f32 / 255.0,
    ); // Slightly darker beige for hover

    // Accent colors - Solarized palette
    pub const ACCENT: Color = Color::from_rgb(
        0x26 as f32 / 255.0,
        0x8b as f32 / 255.0,
        0xd2 as f32 / 255.0,
    ); // #268bd2 - blue (primary accent)

    pub const ACCENT_DIM: Color = Color::from_rgb(
        0x2a as f32 / 255.0,
        0xa1 as f32 / 255.0,
        0x98 as f32 / 255.0,
    ); // #2aa198 - cyan for secondary elements

    pub const ACCENT_GLOW: Color = Color::from_rgba(
        0x26 as f32 / 255.0,
        0x8b as f32 / 255.0,
        0xd2 as f32 / 255.0,
        0.2,
    ); // Blue glow

    pub const SUCCESS: Color = Color::from_rgb(
        0x85 as f32 / 255.0,
        0x99 as f32 / 255.0,
        0x00 as f32 / 255.0,
    ); // #859900 - green

    pub const WARNING: Color = Color::from_rgb(
        0xcb as f32 / 255.0,
        0x4b as f32 / 255.0,
        0x16 as f32 / 255.0,
    ); // #cb4b16 - orange

    pub const ERROR: Color = Color::from_rgb(
        0xdc as f32 / 255.0,
        0x32 as f32 / 255.0,
        0x2f as f32 / 255.0,
    ); // #dc322f - red

    // Text colors - Solarized text tones
    pub const TEXT_PRIMARY: Color = Color::from_rgb(
        0x00 as f32 / 255.0,
        0x2b as f32 / 255.0,
        0x36 as f32 / 255.0,
    ); // #002b36 - dark blue-gray

    pub const TEXT_SECONDARY: Color = Color::from_rgb(
        0x58 as f32 / 255.0,
        0x6e as f32 / 255.0,
        0x75 as f32 / 255.0,
    ); // #586e75 - medium gray

    pub const TEXT_MUTED: Color = Color::from_rgb(
        0x93 as f32 / 255.0,
        0xa1 as f32 / 255.0,
        0xa1 as f32 / 255.0,
    ); // #93a1a1 - light gray

    pub const TEXT_DIM: Color = Color::from_rgb(
        0xbd as f32 / 255.0,
        0xbf as f32 / 255.0,
        0xad as f32 / 255.0,
    ); // #bfbfad - very light gray

    // Border colors
    pub const BORDER: Color = Color::from_rgb(
        0xbd as f32 / 255.0,
        0xbf as f32 / 255.0,
        0xad as f32 / 255.0,
    ); // #bdbfad - subtle border

    pub const BORDER_FOCUSED: Color = Self::ACCENT; // Blue focused border

    // UI Component colors
    pub const HERO_CARD_BG: Color = Self::SURFACE; // Hero card background
    pub const DRAWER_BG: Color = Self::SURFACE; // Sidebar drawer background
    pub const MODAL_BG: Color = Color::from_rgba(0.99, 0.96, 0.89, 0.98); // Modal with slight transparency
    pub const STATUS_DOT_INACTIVE: Color = Color::from_rgb(0.73, 0.76, 0.76); // Gray dot
    pub const STATUS_DOT_ACTIVE: Color = Self::ACCENT; // Blue dot for active state
}
