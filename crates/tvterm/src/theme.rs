use alacritty_terminal::vte::ansi::{NamedColor, Rgb};

/// A terminal color theme.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Theme {
    Dracula,
    Kanagawa,
    TokyoNight,
    CatppuccinMocha,
    SolarizedDark,
    OneDark,
    GruvboxDark,
    AyuDark,
}

impl Theme {
    pub const ALL: &[Theme] = &[
        Theme::Dracula,
        Theme::Kanagawa,
        Theme::TokyoNight,
        Theme::CatppuccinMocha,
        Theme::SolarizedDark,
        Theme::OneDark,
        Theme::GruvboxDark,
        Theme::AyuDark,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Theme::Dracula => "Dracula",
            Theme::Kanagawa => "Kanagawa",
            Theme::TokyoNight => "Tokyo Night",
            Theme::CatppuccinMocha => "Catppuccin Mocha",
            Theme::SolarizedDark => "Solarized Dark",
            Theme::OneDark => "One Dark",
            Theme::GruvboxDark => "Gruvbox Dark",
            Theme::AyuDark => "Ayu Dark",
        }
    }

    pub fn palette(self) -> Palette {
        match self {
            Theme::Dracula => DRACULA,
            Theme::Kanagawa => KANAGAWA,
            Theme::TokyoNight => TOKYO_NIGHT,
            Theme::CatppuccinMocha => CATPPUCCIN_MOCHA,
            Theme::SolarizedDark => SOLARIZED_DARK,
            Theme::OneDark => ONE_DARK,
            Theme::GruvboxDark => GRUVBOX_DARK,
            Theme::AyuDark => AYU_DARK,
        }
    }
}

/// The 16 ANSI colors plus foreground, background, and cursor.
#[derive(Debug, Clone, Copy)]
pub struct Palette {
    pub foreground: Rgb,
    pub background: Rgb,
    pub cursor: Rgb,
    pub black: Rgb,
    pub red: Rgb,
    pub green: Rgb,
    pub yellow: Rgb,
    pub blue: Rgb,
    pub magenta: Rgb,
    pub cyan: Rgb,
    pub white: Rgb,
    pub bright_black: Rgb,
    pub bright_red: Rgb,
    pub bright_green: Rgb,
    pub bright_yellow: Rgb,
    pub bright_blue: Rgb,
    pub bright_magenta: Rgb,
    pub bright_cyan: Rgb,
    pub bright_white: Rgb,
}

impl Palette {
    /// Resolve a named terminal color to an RGB value.
    pub fn resolve_named(self, name: NamedColor) -> Rgb {
        match name {
            NamedColor::Black => self.black,
            NamedColor::Red => self.red,
            NamedColor::Green => self.green,
            NamedColor::Yellow => self.yellow,
            NamedColor::Blue => self.blue,
            NamedColor::Magenta => self.magenta,
            NamedColor::Cyan => self.cyan,
            NamedColor::White => self.white,
            NamedColor::BrightBlack => self.bright_black,
            NamedColor::BrightRed => self.bright_red,
            NamedColor::BrightGreen => self.bright_green,
            NamedColor::BrightYellow => self.bright_yellow,
            NamedColor::BrightBlue => self.bright_blue,
            NamedColor::BrightMagenta => self.bright_magenta,
            NamedColor::BrightCyan => self.bright_cyan,
            NamedColor::BrightWhite => self.bright_white,
            NamedColor::Foreground => self.foreground,
            NamedColor::Background => self.background,
            _ => self.foreground,
        }
    }

    /// Return the background color dimmed by a factor (0.0 = black, 1.0 = original).
    pub fn dimmed_background(self, dim: f32) -> [u8; 3] {
        [
            (self.background.r as f32 * dim) as u8,
            (self.background.g as f32 * dim) as u8,
            (self.background.b as f32 * dim) as u8,
        ]
    }
}

const fn rgb(r: u8, g: u8, b: u8) -> Rgb {
    Rgb { r, g, b }
}

// https://draculatheme.com/contribute
const DRACULA: Palette = Palette {
    foreground: rgb(248, 248, 242),
    background: rgb(40, 42, 54),
    cursor: rgb(248, 248, 242),
    black: rgb(33, 34, 44),
    red: rgb(255, 85, 85),
    green: rgb(80, 250, 123),
    yellow: rgb(241, 250, 140),
    blue: rgb(98, 114, 164),
    magenta: rgb(255, 121, 198),
    cyan: rgb(139, 233, 253),
    white: rgb(248, 248, 242),
    bright_black: rgb(98, 114, 164),
    bright_red: rgb(255, 110, 110),
    bright_green: rgb(105, 255, 148),
    bright_yellow: rgb(255, 255, 165),
    bright_blue: rgb(123, 139, 189),
    bright_magenta: rgb(255, 146, 223),
    bright_cyan: rgb(164, 255, 255),
    bright_white: rgb(255, 255, 255),
};

// https://github.com/rebelot/kanagawa.nvim
const KANAGAWA: Palette = Palette {
    foreground: rgb(220, 215, 186),
    background: rgb(31, 31, 40),
    cursor: rgb(200, 192, 147),
    black: rgb(22, 22, 29),
    red: rgb(195, 64, 67),
    green: rgb(118, 148, 106),
    yellow: rgb(192, 163, 110),
    blue: rgb(126, 156, 216),
    magenta: rgb(149, 127, 184),
    cyan: rgb(106, 149, 137),
    white: rgb(220, 215, 186),
    bright_black: rgb(84, 84, 109),
    bright_red: rgb(231, 115, 118),
    bright_green: rgb(152, 187, 108),
    bright_yellow: rgb(226, 195, 132),
    bright_blue: rgb(126, 156, 216),
    bright_magenta: rgb(210, 126, 153),
    bright_cyan: rgb(127, 180, 202),
    bright_white: rgb(220, 215, 186),
};

// https://github.com/enkia/tokyo-night-vscode-theme
const TOKYO_NIGHT: Palette = Palette {
    foreground: rgb(192, 202, 245),
    background: rgb(26, 27, 38),
    cursor: rgb(192, 202, 245),
    black: rgb(21, 22, 30),
    red: rgb(247, 118, 142),
    green: rgb(158, 206, 106),
    yellow: rgb(224, 175, 104),
    blue: rgb(122, 162, 247),
    magenta: rgb(187, 154, 247),
    cyan: rgb(125, 207, 255),
    white: rgb(169, 177, 214),
    bright_black: rgb(65, 72, 104),
    bright_red: rgb(247, 118, 142),
    bright_green: rgb(158, 206, 106),
    bright_yellow: rgb(224, 175, 104),
    bright_blue: rgb(122, 162, 247),
    bright_magenta: rgb(187, 154, 247),
    bright_cyan: rgb(125, 207, 255),
    bright_white: rgb(192, 202, 245),
};

// https://github.com/catppuccin/catppuccin
const CATPPUCCIN_MOCHA: Palette = Palette {
    foreground: rgb(205, 214, 244),
    background: rgb(30, 30, 46),
    cursor: rgb(245, 224, 220),
    black: rgb(69, 71, 90),
    red: rgb(243, 139, 168),
    green: rgb(166, 227, 161),
    yellow: rgb(249, 226, 175),
    blue: rgb(137, 180, 250),
    magenta: rgb(245, 194, 231),
    cyan: rgb(148, 226, 213),
    white: rgb(186, 194, 222),
    bright_black: rgb(88, 91, 112),
    bright_red: rgb(243, 139, 168),
    bright_green: rgb(166, 227, 161),
    bright_yellow: rgb(249, 226, 175),
    bright_blue: rgb(137, 180, 250),
    bright_magenta: rgb(245, 194, 231),
    bright_cyan: rgb(148, 226, 213),
    bright_white: rgb(205, 214, 244),
};

// https://ethanschoonover.com/solarized/
const SOLARIZED_DARK: Palette = Palette {
    foreground: rgb(131, 148, 150),
    background: rgb(0, 43, 54),
    cursor: rgb(131, 148, 150),
    black: rgb(7, 54, 66),
    red: rgb(220, 50, 47),
    green: rgb(133, 153, 0),
    yellow: rgb(181, 137, 0),
    blue: rgb(38, 139, 210),
    magenta: rgb(211, 54, 130),
    cyan: rgb(42, 161, 152),
    white: rgb(238, 232, 213),
    bright_black: rgb(0, 43, 54),
    bright_red: rgb(203, 75, 22),
    bright_green: rgb(88, 110, 117),
    bright_yellow: rgb(101, 123, 131),
    bright_blue: rgb(131, 148, 150),
    bright_magenta: rgb(108, 113, 196),
    bright_cyan: rgb(147, 161, 161),
    bright_white: rgb(253, 246, 227),
};

// https://github.com/Binaryify/OneDark-Pro
const ONE_DARK: Palette = Palette {
    foreground: rgb(171, 178, 191),
    background: rgb(40, 44, 52),
    cursor: rgb(171, 178, 191),
    black: rgb(40, 44, 52),
    red: rgb(224, 108, 117),
    green: rgb(152, 195, 121),
    yellow: rgb(229, 192, 123),
    blue: rgb(97, 175, 239),
    magenta: rgb(198, 120, 221),
    cyan: rgb(86, 182, 194),
    white: rgb(171, 178, 191),
    bright_black: rgb(92, 99, 112),
    bright_red: rgb(224, 108, 117),
    bright_green: rgb(152, 195, 121),
    bright_yellow: rgb(229, 192, 123),
    bright_blue: rgb(97, 175, 239),
    bright_magenta: rgb(198, 120, 221),
    bright_cyan: rgb(86, 182, 194),
    bright_white: rgb(255, 255, 255),
};

// https://github.com/morhetz/gruvbox
const GRUVBOX_DARK: Palette = Palette {
    foreground: rgb(235, 219, 178),
    background: rgb(40, 40, 40),
    cursor: rgb(235, 219, 178),
    black: rgb(40, 40, 40),
    red: rgb(204, 36, 29),
    green: rgb(152, 151, 26),
    yellow: rgb(215, 153, 33),
    blue: rgb(69, 133, 136),
    magenta: rgb(177, 98, 134),
    cyan: rgb(104, 157, 106),
    white: rgb(168, 153, 132),
    bright_black: rgb(146, 131, 116),
    bright_red: rgb(251, 73, 52),
    bright_green: rgb(184, 187, 38),
    bright_yellow: rgb(250, 189, 47),
    bright_blue: rgb(131, 165, 152),
    bright_magenta: rgb(211, 134, 155),
    bright_cyan: rgb(142, 192, 124),
    bright_white: rgb(235, 219, 178),
};

// https://github.com/ayu-theme/ayu-colors
const AYU_DARK: Palette = Palette {
    foreground: rgb(179, 177, 173),
    background: rgb(11, 14, 20),
    cursor: rgb(232, 183, 82),
    black: rgb(1, 10, 16),
    red: rgb(234, 109, 96),
    green: rgb(145, 180, 99),
    yellow: rgb(232, 183, 82),
    blue: rgb(83, 149, 222),
    magenta: rgb(215, 129, 220),
    cyan: rgb(149, 230, 203),
    white: rgb(192, 191, 188),
    bright_black: rgb(104, 111, 120),
    bright_red: rgb(242, 151, 132),
    bright_green: rgb(172, 202, 136),
    bright_yellow: rgb(255, 215, 131),
    bright_blue: rgb(124, 178, 238),
    bright_magenta: rgb(230, 176, 233),
    bright_cyan: rgb(179, 242, 224),
    bright_white: rgb(255, 255, 255),
};
