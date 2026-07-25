use ratatui::style::{Color, Modifier, Style};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemePreset {
    CloudflareDark,
    CatppuccinMocha,
    TokyoNight,
    Nord,
}

impl ThemePreset {
    pub fn name(&self) -> &'static str {
        match self {
            ThemePreset::CloudflareDark => "Cloudflare Dark",
            ThemePreset::CatppuccinMocha => "Catppuccin Mocha",
            ThemePreset::TokyoNight => "Tokyo Night",
            ThemePreset::Nord => "Nord",
        }
    }

    pub fn all() -> &'static [ThemePreset] {
        &[
            ThemePreset::CloudflareDark,
            ThemePreset::CatppuccinMocha,
            ThemePreset::TokyoNight,
            ThemePreset::Nord,
        ]
    }

    pub fn from_name(name: &str) -> Self {
        let normalized = name.to_lowercase().replace(['-', '_', ' '], "");
        match normalized.as_str() {
            "catppuccinmocha" | "catppuccin" | "mocha" => ThemePreset::CatppuccinMocha,
            "tokyonight" | "tokyo" => ThemePreset::TokyoNight,
            "nord" => ThemePreset::Nord,
            _ => ThemePreset::CloudflareDark,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Theme {
    pub bg: Color,
    pub primary: Color,
    pub secondary: Color,
    pub text: Color,
    pub text_dim: Color,
    pub border: Color,
    pub border_focus: Color,
    pub success: Color,
    pub warning: Color,
    pub error: Color,
    pub info: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self::cloudflare_dark()
    }
}

impl Theme {
    pub fn cloudflare_dark() -> Self {
        Self {
            bg: Color::Reset,
            primary: Color::Rgb(243, 128, 32),     // Cloudflare Orange #F38020
            secondary: Color::Rgb(250, 173, 20),   // Light amber #FAAD14
            text: Color::White,
            text_dim: Color::DarkGray,
            border: Color::Rgb(70, 70, 85),
            border_focus: Color::Rgb(243, 128, 32),
            success: Color::Rgb(80, 200, 120),    // Emerald green
            warning: Color::Rgb(250, 200, 50),     // Yellow
            error: Color::Rgb(240, 80, 80),        // Coral red
            info: Color::Rgb(80, 170, 240),        // Sky blue
        }
    }

    pub fn catppuccin_mocha() -> Self {
        Self {
            bg: Color::Reset,
            primary: Color::Rgb(203, 166, 247),    // Mauve #cba6f7
            secondary: Color::Rgb(245, 194, 231),  // Pink / Flamingo #f5c2e7
            text: Color::Rgb(205, 214, 244),       // Text #cdd6f4
            text_dim: Color::Rgb(108, 112, 134),   // Overlay0 #6c7086
            border: Color::Rgb(69, 71, 90),        // Surface1 #45475a
            border_focus: Color::Rgb(203, 166, 247), // Mauve
            success: Color::Rgb(166, 227, 161),    // Green #a6e3a1
            warning: Color::Rgb(249, 226, 175),    // Yellow #f9e2af
            error: Color::Rgb(243, 139, 168),      // Red #f38ba8
            info: Color::Rgb(137, 180, 250),       // Blue #89b4fa
        }
    }

    pub fn tokyo_night() -> Self {
        Self {
            bg: Color::Reset,
            primary: Color::Rgb(122, 162, 247),    // Cyan/Blue #7aa2f7
            secondary: Color::Rgb(187, 154, 247),  // Purple #bb9af7
            text: Color::Rgb(192, 202, 245),       // Foreground #c0caf5
            text_dim: Color::Rgb(86, 95, 137),     // Dark #565f89
            border: Color::Rgb(65, 72, 104),       // Border #414868
            border_focus: Color::Rgb(122, 162, 247),
            success: Color::Rgb(158, 206, 106),    // Green #9ece6a
            warning: Color::Rgb(224, 175, 104),    // Yellow/Orange #e0af68
            error: Color::Rgb(247, 118, 142),      // Red #f7768e
            info: Color::Rgb(125, 207, 255),       // Light Cyan #7dcfff
        }
    }

    pub fn nord() -> Self {
        Self {
            bg: Color::Reset,
            primary: Color::Rgb(136, 192, 208),    // Frost Nord8 #88c0d0
            secondary: Color::Rgb(129, 161, 193),  // Frost Nord9 #81a1c1
            text: Color::Rgb(236, 239, 244),       // Snow Storm Nord6 #eceff4
            text_dim: Color::Rgb(76, 86, 106),     // Polar Night Nord3 #4c566a
            border: Color::Rgb(59, 66, 82),        // Polar Night Nord1 #3b4252
            border_focus: Color::Rgb(136, 192, 208),
            success: Color::Rgb(163, 190, 140),    // Aurora Green Nord14 #a3be8c
            warning: Color::Rgb(235, 203, 139),    // Aurora Yellow Nord13 #ebcb8b
            error: Color::Rgb(191, 97, 106),       // Aurora Red Nord11 #bf616a
            info: Color::Rgb(94, 129, 172),        // Frost Nord10 #5e81ac
        }
    }

    pub fn from_preset(preset: ThemePreset) -> Self {
        match preset {
            ThemePreset::CloudflareDark => Self::cloudflare_dark(),
            ThemePreset::CatppuccinMocha => Self::catppuccin_mocha(),
            ThemePreset::TokyoNight => Self::tokyo_night(),
            ThemePreset::Nord => Self::nord(),
        }
    }

    pub fn from_name(name: &str) -> Self {
        Self::from_preset(ThemePreset::from_name(name))
    }

    pub fn title(&self) -> Style {
        Style::default().fg(self.primary).add_modifier(Modifier::BOLD)
    }

    pub fn block_border(&self, focused: bool) -> Style {
        if focused {
            Style::default().fg(self.border_focus)
        } else {
            Style::default().fg(self.border)
        }
    }

    pub fn selected_row(&self) -> Style {
        Style::default()
            .bg(Color::Rgb(45, 45, 60))
            .fg(self.secondary)
            .add_modifier(Modifier::BOLD)
    }

    pub fn header_style(&self) -> Style {
        Style::default()
            .fg(self.primary)
            .add_modifier(Modifier::BOLD)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_theme() {
        let theme = Theme::default();
        assert_eq!(theme.primary, Color::Rgb(243, 128, 32));
    }

    #[test]
    fn test_theme_presets() {
        assert_eq!(ThemePreset::all().len(), 4);
        for preset in ThemePreset::all() {
            let theme = Theme::from_preset(*preset);
            assert_ne!(theme.primary, Color::Reset);
            assert_ne!(theme.border_focus, Color::Reset);
            assert!(!preset.name().is_empty());
        }
    }

    #[test]
    fn test_theme_from_name() {
        assert_eq!(ThemePreset::from_name("Cloudflare Dark"), ThemePreset::CloudflareDark);
        assert_eq!(ThemePreset::from_name("Catppuccin Mocha"), ThemePreset::CatppuccinMocha);
        assert_eq!(ThemePreset::from_name("catppuccin"), ThemePreset::CatppuccinMocha);
        assert_eq!(ThemePreset::from_name("Tokyo Night"), ThemePreset::TokyoNight);
        assert_eq!(ThemePreset::from_name("tokyo_night"), ThemePreset::TokyoNight);
        assert_eq!(ThemePreset::from_name("Nord"), ThemePreset::Nord);
        assert_eq!(ThemePreset::from_name("unknown"), ThemePreset::CloudflareDark);

        let t = Theme::from_name("tokyo-night");
        assert_eq!(t.primary, Color::Rgb(122, 162, 247));
    }
}
