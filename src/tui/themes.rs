use ratatui::style::Color;

pub struct OmniCodeTheme {
    pub background: Color,
    pub surface: Color,
    pub border: Color,
    pub accent_green: Color,
    pub accent_purple: Color,
    pub text_primary: Color,
    pub text_secondary: Color,
    pub file_path: Color,
    pub error: Color,
    pub warning: Color,
    pub info: Color,
    pub highlight: Color,
    pub selection: Color,
}

pub fn default_theme() -> OmniCodeTheme {
    OmniCodeTheme {
        background: Color::Rgb(15, 21, 32),
        surface: Color::Rgb(20, 28, 43),
        border: Color::Rgb(30, 42, 58),
        accent_green: Color::Rgb(0, 255, 163),
        accent_purple: Color::Rgb(137, 87, 255),
        text_primary: Color::Rgb(235, 240, 245),
        text_secondary: Color::Rgb(136, 153, 180),
        file_path: Color::Rgb(255, 215, 0),
        error: Color::Rgb(255, 68, 68),
        warning: Color::Rgb(255, 170, 0),
        info: Color::Rgb(0, 170, 255),
        highlight: Color::Rgb(0, 255, 163),
        selection: Color::Rgb(30, 42, 58),
    }
}

pub fn solarized_theme() -> OmniCodeTheme {
    OmniCodeTheme {
        background: Color::Rgb(0, 43, 54),
        surface: Color::Rgb(7, 54, 66),
        border: Color::Rgb(88, 110, 117),
        accent_green: Color::Rgb(133, 153, 0),
        accent_purple: Color::Rgb(108, 113, 196),
        text_primary: Color::Rgb(238, 232, 213),
        text_secondary: Color::Rgb(147, 161, 161),
        file_path: Color::Rgb(181, 137, 0),
        error: Color::Rgb(220, 50, 47),
        warning: Color::Rgb(181, 137, 0),
        info: Color::Rgb(38, 139, 210),
        highlight: Color::Rgb(133, 153, 0),
        selection: Color::Rgb(7, 54, 66),
    }
}

pub fn dracula_theme() -> OmniCodeTheme {
    OmniCodeTheme {
        background: Color::Rgb(40, 42, 54),
        surface: Color::Rgb(68, 71, 90),
        border: Color::Rgb(98, 114, 164),
        accent_green: Color::Rgb(80, 250, 123),
        accent_purple: Color::Rgb(189, 147, 249),
        text_primary: Color::Rgb(248, 248, 242),
        text_secondary: Color::Rgb(98, 114, 164),
        file_path: Color::Rgb(241, 250, 140),
        error: Color::Rgb(255, 85, 85),
        warning: Color::Rgb(255, 184, 108),
        info: Color::Rgb(139, 233, 253),
        highlight: Color::Rgb(80, 250, 123),
        selection: Color::Rgb(68, 71, 90),
    }
}
