use ratatui::style::{Color, Modifier, Style};

pub struct Theme {
    pub bg: Color,
    pub primary: Color,      // Cloudflare Orange
    pub secondary: Color,    // Amber/Gold
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
        Self {
            bg: Color::Reset,
            primary: Color::Rgb(243, 128, 32),     // #F38020 Cloudflare Orange
            secondary: Color::Rgb(250, 173, 20),   // Light amber
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
}

impl Theme {
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
