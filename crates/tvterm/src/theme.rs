use alacritty_terminal::vte::ansi::{NamedColor, Rgb};

/// A terminal color theme.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Theme {
    Dracula,
    GruvboxDark,
    TokyoNight,
    CatppuccinMocha,
}

impl Theme {
    pub const ALL: &[Theme] = &[
        Theme::Dracula,
        Theme::GruvboxDark,
        Theme::TokyoNight,
        Theme::CatppuccinMocha,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Theme::Dracula => "Dracula",
            Theme::GruvboxDark => "Gruvbox Dark",
            Theme::TokyoNight => "Tokyo Night",
            Theme::CatppuccinMocha => "Catppuccin Mocha",
        }
    }

    pub fn palette(self) -> Palette {
        match self {
            Theme::Dracula => DRACULA,
            Theme::GruvboxDark => GRUVBOX_DARK,
            Theme::TokyoNight => TOKYO_NIGHT,
            Theme::CatppuccinMocha => CATPPUCCIN_MOCHA,
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
