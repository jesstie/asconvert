use std::io::{self, Stdout};
use std::time::Duration;

use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap},
    Frame, Terminal,
};

// ─── Soft Colour Palette ─────────────────────────────────────────────────────────
const BG:        Color = Color::Rgb(18, 20, 26);
const SURFACE:   Color = Color::Rgb(26, 29, 38);
const SURFACE2:  Color = Color::Rgb(34, 37, 50);
const SURFACE3:  Color = Color::Rgb(44, 48, 64);
const BORDER:    Color = Color::Rgb(62, 68, 92);
const BORDER_LT: Color = Color::Rgb(80, 88, 115);

// Soft accent palette — pastel-ish
const AMBER:   Color = Color::Rgb(230, 185, 110); // warm gold
const SKY:     Color = Color::Rgb(110, 195, 235); // soft sky blue
const MINT:    Color = Color::Rgb(100, 210, 160); // soft mint
const LAVNDR:  Color = Color::Rgb(175, 145, 235); // soft lavender
const CORAL:   Color = Color::Rgb(235, 130, 120); // soft coral/red
const PEACH:   Color = Color::Rgb(235, 165, 100); // soft peach

const TEXT:       Color = Color::Rgb(200, 205, 220);
const TEXT_DIM:   Color = Color::Rgb(105, 112, 140);
const TEXT_MID:   Color = Color::Rgb(150, 158, 185);
const TEXT_BRIGHT:Color = Color::Rgb(235, 238, 248);

// ASCII table char-category colours (soft)
const TBL_CTRL:  Color = Color::Rgb(200, 130, 130); // control chars — muted coral
const TBL_DIGIT: Color = Color::Rgb(230, 185, 110); // digits — amber
const TBL_UPPER: Color = Color::Rgb(110, 195, 235); // uppercase — sky
const TBL_LOWER: Color = Color::Rgb(100, 210, 160); // lowercase — mint
const TBL_PUNCT: Color = Color::Rgb(175, 145, 235); // punctuation — lavender
const TBL_SPEC:  Color = Color::Rgb(160, 165, 185); // space/del — muted

// ─── ASCII Logo ──────────────────────────────────────────────────────────────────
// const LOGO: &[&str] = &[
//     "                                                                             ",
//     " █████╗ ███████╗ ██████╗ ██████╗ ███╗   ██╗██╗   ██╗███████╗██████╗ ████████╗",
//     "██╔══██╗██╔════╝██╔════╝██╔═══██╗████╗  ██║██║   ██║██╔════╝██╔══██╗╚══██╔══╝",
//     "███████║███████╗██║     ██║   ██║██╔██╗ ██║██║   ██║█████╗  ██████╔╝   ██║   ",
//     "██╔══██║╚════██║██║     ██║   ██║██║╚██╗██║╚██╗ ██╔╝██╔══╝  ██╔══██╗   ██║   ",
//     "██║  ██║███████║╚██████╗╚██████╔╝██║ ╚████║ ╚████╔╝ ███████╗██║  ██║   ██║   ",
//     "╚═╝  ╚═╝╚══════╝ ╚═════╝ ╚═════╝ ╚═╝  ╚═══╝  ╚═══╝  ╚══════╝╚═╝  ╚═╝   ╚═╝   ",
//     "                                                                             ",
// ];

const LOGO: &[&str] = &[
    "   █████████    █████████    █████████                                                      █████   ",
    "  ███░░░░░███  ███░░░░░███  ███░░░░░███                                                    ░░███    ",
    " ░███    ░███ ░███    ░░░  ███     ░░░   ██████  ████████   █████ █████  ██████  ████████  ███████  ",
    " ░███████████ ░░█████████ ░███          ███░░███░░███░░███ ░░███ ░░███  ███░░███░░███░░███░░░███░   ",
    " ░███░░░░░███  ░░░░░░░░███░███         ░███ ░███ ░███ ░███  ░███  ░███ ░███████  ░███ ░░░   ░███    ",
    " ░███    ░███  ███    ░███░░███     ███░███ ░███ ░███ ░███  ░░███ ███  ░███░░░   ░███       ░███ ███",
    " █████   █████░░█████████  ░░█████████ ░░██████  ████ █████  ░░█████   ░░██████  █████      ░░█████ ",
    "░░░░░   ░░░░░  ░░░░░░░░░    ░░░░░░░░░   ░░░░░░  ░░░░ ░░░░░    ░░░░░     ░░░░░░  ░░░░░        ░░░░░  ",   
    "                                                                                                    ",                                                                              
];

// ─── Full ASCII 0–127 table data ─────────────────────────────────────────────────
struct AsciiEntry {
    dec: u8,
    hex: &'static str,
    label: &'static str, // display char or [NAME]
    is_ctrl: bool,
}

const ASCII_TABLE: &[AsciiEntry] = &[
    AsciiEntry { dec: 0,   hex: "00", label: "[NUL]",   is_ctrl: true  },
    AsciiEntry { dec: 1,   hex: "01", label: "[SOH]",   is_ctrl: true  },
    AsciiEntry { dec: 2,   hex: "02", label: "[STX]",   is_ctrl: true  },
    AsciiEntry { dec: 3,   hex: "03", label: "[ETX]",   is_ctrl: true  },
    AsciiEntry { dec: 4,   hex: "04", label: "[EOT]",   is_ctrl: true  },
    AsciiEntry { dec: 5,   hex: "05", label: "[ENQ]",   is_ctrl: true  },
    AsciiEntry { dec: 6,   hex: "06", label: "[ACK]",   is_ctrl: true  },
    AsciiEntry { dec: 7,   hex: "07", label: "[BEL]",   is_ctrl: true  },
    AsciiEntry { dec: 8,   hex: "08", label: "[BS]",    is_ctrl: true  },
    AsciiEntry { dec: 9,   hex: "09", label: "[HT]",    is_ctrl: true  },
    AsciiEntry { dec: 10,  hex: "0A", label: "[LF]",    is_ctrl: true  },
    AsciiEntry { dec: 11,  hex: "0B", label: "[VT]",    is_ctrl: true  },
    AsciiEntry { dec: 12,  hex: "0C", label: "[FF]",    is_ctrl: true  },
    AsciiEntry { dec: 13,  hex: "0D", label: "[CR]",    is_ctrl: true  },
    AsciiEntry { dec: 14,  hex: "0E", label: "[SO]",    is_ctrl: true  },
    AsciiEntry { dec: 15,  hex: "0F", label: "[SI]",    is_ctrl: true  },
    AsciiEntry { dec: 16,  hex: "10", label: "[DLE]",   is_ctrl: true  },
    AsciiEntry { dec: 17,  hex: "11", label: "[DC1]",   is_ctrl: true  },
    AsciiEntry { dec: 18,  hex: "12", label: "[DC2]",   is_ctrl: true  },
    AsciiEntry { dec: 19,  hex: "13", label: "[DC3]",   is_ctrl: true  },
    AsciiEntry { dec: 20,  hex: "14", label: "[DC4]",   is_ctrl: true  },
    AsciiEntry { dec: 21,  hex: "15", label: "[NAK]",   is_ctrl: true  },
    AsciiEntry { dec: 22,  hex: "16", label: "[SYN]",   is_ctrl: true  },
    AsciiEntry { dec: 23,  hex: "17", label: "[ETB]",   is_ctrl: true  },
    AsciiEntry { dec: 24,  hex: "18", label: "[CAN]",   is_ctrl: true  },
    AsciiEntry { dec: 25,  hex: "19", label: "[EM]",    is_ctrl: true  },
    AsciiEntry { dec: 26,  hex: "1A", label: "[SUB]",   is_ctrl: true  },
    AsciiEntry { dec: 27,  hex: "1B", label: "[ESC]",   is_ctrl: true  },
    AsciiEntry { dec: 28,  hex: "1C", label: "[FS]",    is_ctrl: true  },
    AsciiEntry { dec: 29,  hex: "1D", label: "[GS]",    is_ctrl: true  },
    AsciiEntry { dec: 30,  hex: "1E", label: "[RS]",    is_ctrl: true  },
    AsciiEntry { dec: 31,  hex: "1F", label: "[US]",    is_ctrl: true  },
    AsciiEntry { dec: 32,  hex: "20", label: "[SPC]",   is_ctrl: false },
    AsciiEntry { dec: 33,  hex: "21", label: "!",       is_ctrl: false },
    AsciiEntry { dec: 34,  hex: "22", label: "\"",      is_ctrl: false },
    AsciiEntry { dec: 35,  hex: "23", label: "#",       is_ctrl: false },
    AsciiEntry { dec: 36,  hex: "24", label: "$",       is_ctrl: false },
    AsciiEntry { dec: 37,  hex: "25", label: "%",       is_ctrl: false },
    AsciiEntry { dec: 38,  hex: "26", label: "&",       is_ctrl: false },
    AsciiEntry { dec: 39,  hex: "27", label: "'",       is_ctrl: false },
    AsciiEntry { dec: 40,  hex: "28", label: "(",       is_ctrl: false },
    AsciiEntry { dec: 41,  hex: "29", label: ")",       is_ctrl: false },
    AsciiEntry { dec: 42,  hex: "2A", label: "*",       is_ctrl: false },
    AsciiEntry { dec: 43,  hex: "2B", label: "+",       is_ctrl: false },
    AsciiEntry { dec: 44,  hex: "2C", label: ",",       is_ctrl: false },
    AsciiEntry { dec: 45,  hex: "2D", label: "-",       is_ctrl: false },
    AsciiEntry { dec: 46,  hex: "2E", label: ".",       is_ctrl: false },
    AsciiEntry { dec: 47,  hex: "2F", label: "/",       is_ctrl: false },
    AsciiEntry { dec: 48,  hex: "30", label: "0",       is_ctrl: false },
    AsciiEntry { dec: 49,  hex: "31", label: "1",       is_ctrl: false },
    AsciiEntry { dec: 50,  hex: "32", label: "2",       is_ctrl: false },
    AsciiEntry { dec: 51,  hex: "33", label: "3",       is_ctrl: false },
    AsciiEntry { dec: 52,  hex: "34", label: "4",       is_ctrl: false },
    AsciiEntry { dec: 53,  hex: "35", label: "5",       is_ctrl: false },
    AsciiEntry { dec: 54,  hex: "36", label: "6",       is_ctrl: false },
    AsciiEntry { dec: 55,  hex: "37", label: "7",       is_ctrl: false },
    AsciiEntry { dec: 56,  hex: "38", label: "8",       is_ctrl: false },
    AsciiEntry { dec: 57,  hex: "39", label: "9",       is_ctrl: false },
    AsciiEntry { dec: 58,  hex: "3A", label: ":",       is_ctrl: false },
    AsciiEntry { dec: 59,  hex: "3B", label: ";",       is_ctrl: false },
    AsciiEntry { dec: 60,  hex: "3C", label: "<",       is_ctrl: false },
    AsciiEntry { dec: 61,  hex: "3D", label: "=",       is_ctrl: false },
    AsciiEntry { dec: 62,  hex: "3E", label: ">",       is_ctrl: false },
    AsciiEntry { dec: 63,  hex: "3F", label: "?",       is_ctrl: false },
    AsciiEntry { dec: 64,  hex: "40", label: "@",       is_ctrl: false },
    AsciiEntry { dec: 65,  hex: "41", label: "A",       is_ctrl: false },
    AsciiEntry { dec: 66,  hex: "42", label: "B",       is_ctrl: false },
    AsciiEntry { dec: 67,  hex: "43", label: "C",       is_ctrl: false },
    AsciiEntry { dec: 68,  hex: "44", label: "D",       is_ctrl: false },
    AsciiEntry { dec: 69,  hex: "45", label: "E",       is_ctrl: false },
    AsciiEntry { dec: 70,  hex: "46", label: "F",       is_ctrl: false },
    AsciiEntry { dec: 71,  hex: "47", label: "G",       is_ctrl: false },
    AsciiEntry { dec: 72,  hex: "48", label: "H",       is_ctrl: false },
    AsciiEntry { dec: 73,  hex: "49", label: "I",       is_ctrl: false },
    AsciiEntry { dec: 74,  hex: "4A", label: "J",       is_ctrl: false },
    AsciiEntry { dec: 75,  hex: "4B", label: "K",       is_ctrl: false },
    AsciiEntry { dec: 76,  hex: "4C", label: "L",       is_ctrl: false },
    AsciiEntry { dec: 77,  hex: "4D", label: "M",       is_ctrl: false },
    AsciiEntry { dec: 78,  hex: "4E", label: "N",       is_ctrl: false },
    AsciiEntry { dec: 79,  hex: "4F", label: "O",       is_ctrl: false },
    AsciiEntry { dec: 80,  hex: "50", label: "P",       is_ctrl: false },
    AsciiEntry { dec: 81,  hex: "51", label: "Q",       is_ctrl: false },
    AsciiEntry { dec: 82,  hex: "52", label: "R",       is_ctrl: false },
    AsciiEntry { dec: 83,  hex: "53", label: "S",       is_ctrl: false },
    AsciiEntry { dec: 84,  hex: "54", label: "T",       is_ctrl: false },
    AsciiEntry { dec: 85,  hex: "55", label: "U",       is_ctrl: false },
    AsciiEntry { dec: 86,  hex: "56", label: "V",       is_ctrl: false },
    AsciiEntry { dec: 87,  hex: "57", label: "W",       is_ctrl: false },
    AsciiEntry { dec: 88,  hex: "58", label: "X",       is_ctrl: false },
    AsciiEntry { dec: 89,  hex: "59", label: "Y",       is_ctrl: false },
    AsciiEntry { dec: 90,  hex: "5A", label: "Z",       is_ctrl: false },
    AsciiEntry { dec: 91,  hex: "5B", label: "[",       is_ctrl: false },
    AsciiEntry { dec: 92,  hex: "5C", label: "\\",      is_ctrl: false },
    AsciiEntry { dec: 93,  hex: "5D", label: "]",       is_ctrl: false },
    AsciiEntry { dec: 94,  hex: "5E", label: "^",       is_ctrl: false },
    AsciiEntry { dec: 95,  hex: "5F", label: "_",       is_ctrl: false },
    AsciiEntry { dec: 96,  hex: "60", label: "`",       is_ctrl: false },
    AsciiEntry { dec: 97,  hex: "61", label: "a",       is_ctrl: false },
    AsciiEntry { dec: 98,  hex: "62", label: "b",       is_ctrl: false },
    AsciiEntry { dec: 99,  hex: "63", label: "c",       is_ctrl: false },
    AsciiEntry { dec: 100, hex: "64", label: "d",       is_ctrl: false },
    AsciiEntry { dec: 101, hex: "65", label: "e",       is_ctrl: false },
    AsciiEntry { dec: 102, hex: "66", label: "f",       is_ctrl: false },
    AsciiEntry { dec: 103, hex: "67", label: "g",       is_ctrl: false },
    AsciiEntry { dec: 104, hex: "68", label: "h",       is_ctrl: false },
    AsciiEntry { dec: 105, hex: "69", label: "i",       is_ctrl: false },
    AsciiEntry { dec: 106, hex: "6A", label: "j",       is_ctrl: false },
    AsciiEntry { dec: 107, hex: "6B", label: "k",       is_ctrl: false },
    AsciiEntry { dec: 108, hex: "6C", label: "l",       is_ctrl: false },
    AsciiEntry { dec: 109, hex: "6D", label: "m",       is_ctrl: false },
    AsciiEntry { dec: 110, hex: "6E", label: "n",       is_ctrl: false },
    AsciiEntry { dec: 111, hex: "6F", label: "o",       is_ctrl: false },
    AsciiEntry { dec: 112, hex: "70", label: "p",       is_ctrl: false },
    AsciiEntry { dec: 113, hex: "71", label: "q",       is_ctrl: false },
    AsciiEntry { dec: 114, hex: "72", label: "r",       is_ctrl: false },
    AsciiEntry { dec: 115, hex: "73", label: "s",       is_ctrl: false },
    AsciiEntry { dec: 116, hex: "74", label: "t",       is_ctrl: false },
    AsciiEntry { dec: 117, hex: "75", label: "u",       is_ctrl: false },
    AsciiEntry { dec: 118, hex: "76", label: "v",       is_ctrl: false },
    AsciiEntry { dec: 119, hex: "77", label: "w",       is_ctrl: false },
    AsciiEntry { dec: 120, hex: "78", label: "x",       is_ctrl: false },
    AsciiEntry { dec: 121, hex: "79", label: "y",       is_ctrl: false },
    AsciiEntry { dec: 122, hex: "7A", label: "z",       is_ctrl: false },
    AsciiEntry { dec: 123, hex: "7B", label: "{",       is_ctrl: false },
    AsciiEntry { dec: 124, hex: "7C", label: "|",       is_ctrl: false },
    AsciiEntry { dec: 125, hex: "7D", label: "}",       is_ctrl: false },
    AsciiEntry { dec: 126, hex: "7E", label: "~",       is_ctrl: false },
    AsciiEntry { dec: 127, hex: "7F", label: "[DEL]",   is_ctrl: true  },
];

fn entry_char_color(e: &AsciiEntry) -> Color {
    if e.is_ctrl             { TBL_CTRL  }
    else if e.dec == 32      { TBL_SPEC  }
    else if e.dec == 127     { TBL_CTRL  }
    else if e.dec >= 48 && e.dec <= 57  { TBL_DIGIT }
    else if e.dec >= 65 && e.dec <= 90  { TBL_UPPER }
    else if e.dec >= 97 && e.dec <= 122 { TBL_LOWER }
    else                     { TBL_PUNCT }
}

// ─── Mode ────────────────────────────────────────────────────────────────────────
#[derive(Clone, PartialEq)]
enum Mode {
    TextToDecimal,
    TextToHex,
    DecimalToText,
    HexToText,
}

impl Mode {
    fn label(&self) -> &'static str {
        match self {
            Mode::TextToDecimal => "Text  →  Decimal",
            Mode::TextToHex     => "Text  →  Hex",
            Mode::DecimalToText => "Decimal  →  Text",
            Mode::HexToText     => "Hex  →  Text",
        }
    }
    fn input_hint(&self) -> &'static str {
        match self {
            Mode::TextToDecimal | Mode::TextToHex => "Type any text…",
            Mode::DecimalToText => "e.g.  72 101 108  or  72,101,108",
            Mode::HexToText     => "e.g.  48 65 6C  or  48,65,6C",
        }
    }
    fn accent(&self) -> Color {
        match self {
            Mode::TextToDecimal => AMBER,
            Mode::TextToHex     => SKY,
            Mode::DecimalToText => MINT,
            Mode::HexToText     => LAVNDR,
        }
    }
    fn icon(&self) -> &'static str {
        match self { Mode::TextToDecimal => "◆", Mode::TextToHex => "◈", Mode::DecimalToText => "◉", Mode::HexToText => "◎" }
    }
    fn all() -> [Mode; 4] {
        [Mode::TextToDecimal, Mode::TextToHex, Mode::DecimalToText, Mode::HexToText]
    }
    fn index(&self) -> usize {
        match self { Mode::TextToDecimal => 0, Mode::TextToHex => 1, Mode::DecimalToText => 2, Mode::HexToText => 3 }
    }
    fn from_index(i: usize) -> Mode {
        match i { 0 => Mode::TextToDecimal, 1 => Mode::TextToHex, 2 => Mode::DecimalToText, _ => Mode::HexToText }
    }
}

// ─── Separator ───────────────────────────────────────────────────────────────────
#[derive(Clone, PartialEq)]
enum Separator { Space, Comma, Newline }

impl Separator {
    fn label(&self) -> &'static str { match self { Separator::Space => "Space", Separator::Comma => "Comma", Separator::Newline => "Newline" } }
    fn next(&self) -> Separator { match self { Separator::Space => Separator::Comma, Separator::Comma => Separator::Newline, Separator::Newline => Separator::Space } }
    fn char(&self) -> &'static str { match self { Separator::Space => " ", Separator::Comma => ",", Separator::Newline => "\n" } }
}

// ─── App State ───────────────────────────────────────────────────────────────────
struct App {
    mode: Mode,
    input: String,
    output: String,
    error: Option<String>,
    show_help: bool,
    show_table: bool,
    table_scroll: usize,
    cursor_visible: bool,
    tick: u64,
    separator: Separator,
}

impl App {
    fn new() -> Self {
        Self {
            mode: Mode::TextToDecimal,
            input: String::new(),
            output: String::new(),
            error: None,
            show_help: false,
            show_table: false,
            table_scroll: 0,
            cursor_visible: true,
            tick: 0,
            separator: Separator::Space,
        }
    }

    fn next_mode(&mut self) { self.mode = Mode::from_index((self.mode.index() + 1) % 4); self.recalculate(); }
    fn prev_mode(&mut self) { self.mode = Mode::from_index((self.mode.index() + 4 - 1) % 4); self.recalculate(); }
    fn push_char(&mut self, c: char) { self.input.push(c); self.recalculate(); }
    fn pop_char(&mut self) { self.input.pop(); self.recalculate(); }
    fn clear_input(&mut self) { self.input.clear(); self.output.clear(); self.error = None; }
    fn cycle_separator(&mut self) { self.separator = self.separator.next(); self.recalculate(); }

    fn table_scroll_down(&mut self, n: usize) {
        // 128 entries, 4 cols, 32 rows — but we render row-by-row
        let max = ASCII_TABLE.len().saturating_sub(1);
        self.table_scroll = (self.table_scroll + n).min(max);
    }
    fn table_scroll_up(&mut self, n: usize) {
        self.table_scroll = self.table_scroll.saturating_sub(n);
    }

    fn recalculate(&mut self) {
        self.error = None;
        if self.input.is_empty() { self.output.clear(); return; }
        match &self.mode {
            Mode::TextToDecimal => {
                let sep = self.separator.char();
                self.output = self.input.bytes().map(|b| b.to_string()).collect::<Vec<_>>().join(sep);
            }
            Mode::TextToHex => {
                let sep = self.separator.char();
                self.output = self.input.bytes().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(sep);
            }
            Mode::DecimalToText => {
                let cleaned = self.input.replace(',', " ");
                let mut result = String::new();
                for part in cleaned.split_whitespace() {
                    match part.parse::<u8>() {
                        Ok(b)  => result.push(b as char),
                        Err(_) => { self.error = Some(format!("Invalid decimal: '{}'", part)); self.output.clear(); return; }
                    }
                }
                self.output = result;
            }
            Mode::HexToText => {
                let cleaned = self.input.replace(',', " ");
                let mut result = String::new();
                for part in cleaned.split_whitespace() {
                    match u8::from_str_radix(part, 16) {
                        Ok(b)  => result.push(b as char),
                        Err(_) => { self.error = Some(format!("Invalid hex: '{}'", part)); self.output.clear(); return; }
                    }
                }
                self.output = result;
            }
        }
    }
}

// ─── Main ────────────────────────────────────────────────────────────────────────
fn main() -> io::Result<()> {
    let mut terminal = setup_terminal()?;
    let mut app = App::new();
    let tick_rate = Duration::from_millis(80);

    loop {
        app.tick = app.tick.wrapping_add(1);
        if app.tick % 6 == 0 { app.cursor_visible = !app.cursor_visible; }

        terminal.draw(|f| render(f, &mut app))?;

        if event::poll(tick_rate)? {
            if let Event::Key(key) = event::read()? {
                // Ctrl combos
                if key.modifiers == KeyModifiers::CONTROL {
                    match key.code {
                        KeyCode::Char('c') | KeyCode::Char('q') => break,
                        KeyCode::Char('l') => app.clear_input(),
                        KeyCode::Char('s') => app.cycle_separator(),
                        KeyCode::Char('w') => {
                            let t = app.input.trim_end().to_string();
                            app.input = match t.rfind(' ') { Some(i) => t[..=i].to_string(), None => String::new() };
                            app.recalculate();
                        }
                        _ => {}
                    }
                    continue;
                }

                // Table scroll
                if app.show_table {
                    match key.code {
                        KeyCode::Down | KeyCode::Char('j') => { app.table_scroll_down(1); continue; }
                        KeyCode::Up   | KeyCode::Char('k') => { app.table_scroll_up(1);   continue; }
                        KeyCode::PageDown => { app.table_scroll_down(10); continue; }
                        KeyCode::PageUp   => { app.table_scroll_up(10);   continue; }
                        KeyCode::Home => { app.table_scroll = 0; continue; }
                        KeyCode::End  => { app.table_scroll = ASCII_TABLE.len() - 1; continue; }
                        KeyCode::Esc | KeyCode::F(2) => { app.show_table = false; continue; }
                        _ => {}
                    }
                }

                match key.code {
                    KeyCode::Esc => {
                        if app.show_help || app.show_table { app.show_help = false; app.show_table = false; }
                        else { break; }
                    }
                    KeyCode::F(1) => { app.show_help = !app.show_help; app.show_table = false; }
                    KeyCode::F(2) => { app.show_table = !app.show_table; app.show_help = false; }
                    KeyCode::Tab      => app.next_mode(),
                    KeyCode::BackTab  => app.prev_mode(),
                    KeyCode::Left     => app.prev_mode(),
                    KeyCode::Right    => app.next_mode(),
                    KeyCode::Backspace => app.pop_char(),
                    KeyCode::Delete   => app.clear_input(),
                    KeyCode::Enter    => app.push_char('\n'),
                    KeyCode::Char(c)  => app.push_char(c),
                    _ => {}
                }
            }
        }
    }

    restore_terminal(terminal)?;
    Ok(())
}

fn setup_terminal() -> io::Result<Terminal<CrosstermBackend<Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    Ok(Terminal::new(CrosstermBackend::new(stdout))?)
}

fn restore_terminal(mut terminal: Terminal<CrosstermBackend<Stdout>>) -> io::Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

// ─── Root Renderer ───────────────────────────────────────────────────────────────
fn render(f: &mut Frame, app: &mut App) {
    let area = f.area();
    f.render_widget(Block::default().style(Style::default().bg(BG)), area);

    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(11),  // logo banner
            Constraint::Length(3),  // mode tabs
            Constraint::Min(10),    // input + output
            Constraint::Length(1),  // status bar
        ])
        .margin(1)
        .split(area);

    render_logo(f, app, root[0]);
    render_mode_tabs(f, app, root[1]);

    let body = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(48), Constraint::Percentage(52)])
        .split(root[2]);

    render_input_box(f, app, body[0]);
    render_output_box(f, app, body[1]);
    render_status_bar(f, app, root[3]);

    if app.show_help  { render_help_popup(f, area); }
    if app.show_table { render_ascii_table_full(f, app, area); }
}

// ─── Logo Banner ─────────────────────────────────────────────────────────────────
fn render_logo(f: &mut Frame, app: &App, area: Rect) {
    let accent = PEACH;

    let mut lines: Vec<Line> = LOGO
    .iter()
    .enumerate()
    .map(|(row_idx, &row)| {

        let total_rows = LOGO.len().max(1);

        // interpolation 0.0 → 1.0
        let t = row_idx as f32 / (total_rows - 1) as f32;

        // TOP color
        let top = (255.0, 205.0, 120.0);

        // BOTTOM color
        let bottom = (210.0, 120.0, 80.0);

        // interpolate
        let r = (top.0 * (1.0 - t) + bottom.0 * t) as u8;
        let g = (top.1 * (1.0 - t) + bottom.1 * t) as u8;
        let b = (top.2 * (1.0 - t) + bottom.2 * t) as u8;

        let color = Color::Rgb(r, g, b);

        let spans: Vec<Span> = row
            .chars()
            .map(|c| {
                Span::styled(
                    c.to_string(),
                    Style::default()
                        .fg(color)
                        .add_modifier(Modifier::BOLD),
                )
            })
            .collect();

        Line::from(spans)
    })
    .collect();

    // Tagline below logo
    lines.push(Line::from(vec![
        Span::styled("  ASCII ↔ Text converter  ·  ", Style::default().fg(TEXT_DIM)),
        Span::styled("Dec", Style::default().fg(AMBER)),
        Span::styled(" · ", Style::default().fg(TEXT_DIM)),
        Span::styled("Hex", Style::default().fg(SKY)),
        Span::styled(" · ", Style::default().fg(TEXT_DIM)),
        Span::styled("Bidirectional", Style::default().fg(MINT)),
        Span::styled("  ·  by Jesstie", Style::default().fg(TEXT_DIM)),
    ]));

    f.render_widget(Paragraph::new(lines).alignment(Alignment::Center), area);
}

// ─── Mode Tabs ───────────────────────────────────────────────────────────────────
fn render_mode_tabs(f: &mut Frame, app: &App, area: Rect) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(25); 4])
        .split(area);

    for mode in Mode::all().iter() {
        let is_active = mode.index() == app.mode.index();
        let accent = mode.accent();
        let (bg, fg, btype) = if is_active {
            (SURFACE2, accent, BorderType::Thick)
        } else {
            (SURFACE, TEXT_DIM, BorderType::Rounded)
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(btype)
            .border_style(Style::default().fg(if is_active { accent } else { BORDER }))
            .style(Style::default().bg(bg));

        let label = Line::from(vec![
            Span::styled(format!("{} ", mode.icon()), Style::default().fg(if is_active { accent } else { TEXT_DIM }).bg(bg)),
            Span::styled(mode.label(), Style::default().fg(fg).bg(bg).add_modifier(if is_active { Modifier::BOLD } else { Modifier::empty() })),
        ]);

        f.render_widget(Paragraph::new(label).alignment(Alignment::Center).block(block), cols[mode.index()]);
    }
}

// ─── Input Box ───────────────────────────────────────────────────────────────────
fn render_input_box(f: &mut Frame, app: &App, area: Rect) {
    let accent = app.mode.accent();
    let has_input = !app.input.is_empty();

    let block = Block::default()
        .title(Line::from(vec![
            Span::styled(" INPUT ", Style::default().fg(BG).bg(accent).add_modifier(Modifier::BOLD)),
            Span::styled(format!(" {} chars ", app.input.len()), Style::default().fg(TEXT_DIM)),
            Span::styled(format!(" sep:{} ", app.separator.label()), Style::default().fg(TEXT_DIM)),
        ]))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(accent))
        .style(Style::default().bg(SURFACE));

    let inner = block.inner(area).inner(Margin { horizontal: 1, vertical: 0 });
    f.render_widget(block, area);

    let (display, style) = if has_input {
        let cursor = if app.cursor_visible { "▌" } else { " " };
        (format!("{}{}", app.input, cursor), Style::default().fg(TEXT_BRIGHT))
    } else {
        (format!("  {}", app.mode.input_hint()), Style::default().fg(TEXT_DIM).add_modifier(Modifier::ITALIC))
    };

    f.render_widget(Paragraph::new(display).style(style).wrap(Wrap { trim: false }), inner);
}

// ─── Output Box ──────────────────────────────────────────────────────────────────
fn render_output_box(f: &mut Frame, app: &App, area: Rect) {
    let has_output = !app.output.is_empty();
    let has_error  = app.error.is_some();

    let (border_col, title_bg, title_lbl) = if has_error {
        (CORAL, CORAL, " ERROR ")
    } else if has_output {
        (MINT, MINT, " OUTPUT ")
    } else {
        (BORDER, BORDER_LT, " OUTPUT ")
    };

    let char_count = if has_output && !has_error { format!(" {} chars ", app.output.len()) } else { String::new() };

    let block = Block::default()
        .title(Line::from(vec![
            Span::styled(title_lbl, Style::default().fg(BG).bg(title_bg).add_modifier(Modifier::BOLD)),
            Span::styled(char_count, Style::default().fg(TEXT_DIM)),
        ]))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_col))
        .style(Style::default().bg(SURFACE));

    let inner = block.inner(area).inner(Margin { horizontal: 1, vertical: 0 });
    f.render_widget(block, area);

    let content = if let Some(err) = &app.error {
        Paragraph::new(format!("✕  {}", err)).style(Style::default().fg(CORAL)).wrap(Wrap { trim: false })
    } else if has_output {
        Paragraph::new(app.output.clone()).style(Style::default().fg(TEXT_BRIGHT)).wrap(Wrap { trim: false })
    } else {
        Paragraph::new("  Output will appear here…").style(Style::default().fg(TEXT_DIM).add_modifier(Modifier::ITALIC))
    };

    f.render_widget(content, inner);
}

// ─── Status Bar ──────────────────────────────────────────────────────────────────
fn render_status_bar(f: &mut Frame, app: &App, area: Rect) {
    let ac = app.mode.accent();
    let left = vec![
        Span::styled(" ◆ ", Style::default().fg(ac)),
        Span::styled("Tab/←→", Style::default().fg(ac)), Span::styled(" Mode  ", Style::default().fg(TEXT_DIM)),
        Span::styled("Alt+S",  Style::default().fg(ac)), Span::styled(" Sep  ",  Style::default().fg(TEXT_DIM)),
        Span::styled("Del",    Style::default().fg(ac)), Span::styled(" Clear  ", Style::default().fg(TEXT_DIM)),
    ];
    let right = vec![
        Span::styled("F2", Style::default().fg(SKY)),   Span::styled(" ASCII Table  ", Style::default().fg(TEXT_DIM)),
        Span::styled("F1", Style::default().fg(SKY)),   Span::styled(" Help  ", Style::default().fg(TEXT_DIM)),
        Span::styled("Esc", Style::default().fg(CORAL)),Span::styled(" Quit ", Style::default().fg(TEXT_DIM)),
    ];

    let halves = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(area);

    f.render_widget(Paragraph::new(Line::from(left)).style(Style::default().bg(SURFACE2)), halves[0]);
    f.render_widget(Paragraph::new(Line::from(right)).alignment(Alignment::Right).style(Style::default().bg(SURFACE2)), halves[1]);
}

// ─── Full ASCII Table (0–127) ────────────────────────────────────────────────────
fn render_ascii_table_full(f: &mut Frame, app: &mut App, area: Rect) {
    // popup dimensions
    let w = area.width.saturating_sub(2).min(104);
    let h = area.height.saturating_sub(2).min(35);
    let x = area.x + area.width.saturating_sub(w) / 2;
    let y = area.y + area.height.saturating_sub(h) / 2;
    let popup = Rect::new(x, y, w, h);

    f.render_widget(Clear, popup);

    let outer_block = Block::default()
        .title(Line::from(vec![
            Span::styled(" ◈ ", Style::default().fg(SKY)),
            Span::styled("ASCII Reference Table", Style::default().fg(TEXT_BRIGHT).add_modifier(Modifier::BOLD)),
            Span::styled("  0 – 127 ", Style::default().fg(TEXT_DIM)),
        ]))
        .title_alignment(Alignment::Left)
        .title_bottom(Line::from(vec![
            Span::styled(" ↑↓ / j k ", Style::default().fg(AMBER)),
            Span::styled("scroll  ", Style::default().fg(TEXT_DIM)),
            Span::styled("PgUp PgDn ", Style::default().fg(AMBER)),
            Span::styled("fast  ", Style::default().fg(TEXT_DIM)),
            Span::styled("Home End ", Style::default().fg(AMBER)),
            Span::styled("jump  ", Style::default().fg(TEXT_DIM)),
            Span::styled("F2 / Esc ", Style::default().fg(CORAL)),
            Span::styled("close ", Style::default().fg(TEXT_DIM)),
        ]))
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(SKY))
        .style(Style::default().bg(SURFACE));

    let table_area = outer_block.inner(popup);
    f.render_widget(outer_block, popup);

    // Reserve space for scrollbar on the right
    let table_width = table_area.width.saturating_sub(1);
    let table_content_area = Rect {
        x: table_area.x,
        y: table_area.y,
        width: table_width,
        height: table_area.height,
    };

    // Divide into 4 columns
    let col_w = [Constraint::Percentage(25), Constraint::Percentage(25), Constraint::Percentage(25), Constraint::Percentage(25)];
    let cols = Layout::default().direction(Direction::Horizontal).constraints(col_w).split(table_content_area);

    // 128 entries, 4 columns of 32 rows each
    let rows_per_col: usize = 32;
    let visible_rows = (table_area.height as usize).saturating_sub(4); // minus header row
    let scroll = app.table_scroll.min(rows_per_col.saturating_sub(visible_rows));
    app.table_scroll = scroll;

    let col_header_style = Style::default().fg(TEXT_BRIGHT).add_modifier(Modifier::BOLD).bg(SURFACE3);
    let sep_style        = Style::default().fg(BORDER_LT);

    for col_idx in 0..4usize {
        let base = col_idx * rows_per_col; // 0, 32, 64, 96

        let mut lines: Vec<Line> = Vec::new();

        // ── column header ──
        lines.push(Line::from(vec![
            Span::styled(" Dec ", col_header_style),
            Span::styled("│", sep_style),
            Span::styled(" Hex ", col_header_style),
            Span::styled("│", sep_style),
            Span::styled(" Char      ", col_header_style),
        ]));
        // header underline
        lines.push(Line::from(vec![
            Span::styled("─────┼─────┼───────────", sep_style),
        ]));

        // ── data rows ──
        for row_offset in scroll..(scroll + visible_rows) {
            let entry_idx = base + row_offset;
            if entry_idx >= ASCII_TABLE.len() { break; }
            let e = &ASCII_TABLE[entry_idx];

            // alternating row bg via style — we can't truly set bg per line in
            // Paragraph, so we use a dim bar for even rows via a dimmed fg on dec
            let is_even = row_offset % 2 == 0;
            let dec_col  = if is_even { TEXT_MID   } else { TEXT_DIM   };
            let hex_col  = if is_even { TEXT_MID   } else { TEXT_DIM   };
            let char_col = entry_char_color(e);

            lines.push(Line::from(vec![
                Span::styled(format!(" {:>3} ", e.dec),    Style::default().fg(dec_col)),
                Span::styled("│", sep_style),
                Span::styled(format!(" {:>2}  ", e.hex),   Style::default().fg(hex_col)),
                Span::styled("│", sep_style),
                Span::styled(format!(" {:<9}", e.label),   Style::default().fg(char_col).add_modifier(if e.is_ctrl { Modifier::ITALIC } else { Modifier::empty() })),
            ]));
        }

        // Render this column
        let col_block = Block::default()
            .borders(if col_idx == 0 { Borders::RIGHT } else if col_idx == 3 { Borders::LEFT } else { Borders::LEFT | Borders::RIGHT })
            .border_style(Style::default().fg(BORDER))
            .style(Style::default().bg(SURFACE));

        let col_inner = col_block.inner(cols[col_idx]);
        f.render_widget(col_block, cols[col_idx]);
        f.render_widget(Paragraph::new(lines), col_inner);
    }

    // Scrollbar on the right edge
    let total_rows = rows_per_col;
    let mut sb_state = ScrollbarState::default()
        .content_length(total_rows - (visible_rows - if visible_rows % 2 == 0 { 2 } else { 1 }))
        .position(scroll);
    let sb_area = Rect {
        x: table_area.right().saturating_sub(1),
        y: table_area.y,
        width: 1,
        height: table_area.height,
    };
    f.render_stateful_widget(
        Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .style(Style::default().fg(BORDER_LT)),
        sb_area,
        &mut sb_state,
    );
}

// ─── Help Popup ──────────────────────────────────────────────────────────────────
fn render_help_popup(f: &mut Frame, area: Rect) {
    let w = 54u16.min(area.width.saturating_sub(4));
    let h = 24u16.min(area.height.saturating_sub(2));
    let x = area.x + area.width.saturating_sub(w) / 2;
    let y = area.y + area.height.saturating_sub(h) / 2;
    let popup = Rect::new(x, y, w, h);

    f.render_widget(Clear, popup);

    let block = Block::default()
        .title(Line::from(vec![
            Span::styled(" ◇ ", Style::default().fg(AMBER)),
            Span::styled("Keyboard Shortcuts", Style::default().fg(TEXT_BRIGHT).add_modifier(Modifier::BOLD)),
            Span::raw(" "),
        ]))
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(AMBER))
        .style(Style::default().bg(SURFACE2));

    let inner = block.inner(popup);
    f.render_widget(block, popup);

    let sec  = |s: &'static str| Line::from(vec![Span::styled(format!("  {}", s), Style::default().fg(TEXT_DIM).add_modifier(Modifier::BOLD))]);
    let row  = |k: &'static str, d: &'static str| Line::from(vec![
        Span::raw("  "),
        Span::styled(format!("{:<16}", k), Style::default().fg(AMBER).add_modifier(Modifier::BOLD)),
        Span::styled(d, Style::default().fg(TEXT)),
    ]);

    let lines = vec![
        Line::from(""),
        sec("Navigation"),
        row("Tab / →",       "Next mode"),
        row("Shift+Tab / ←", "Previous mode"),
        row("Alt+S",         "Cycle output separator"),
        Line::from(""),
        sec("Editing"),
        row("Any key",       "Type input"),
        row("Backspace",     "Delete last character"),
        row("Ctrl+W",        "Delete last word"),
        row("Delete/Ctrl+L", "Clear all"),
        Line::from(""),
        sec("Input format for codes"),
        Line::from(vec![Span::raw("  "), Span::styled("Dec: ", Style::default().fg(MINT)),   Span::styled("72 101 108  or  72,101,108", Style::default().fg(TEXT))]),
        Line::from(vec![Span::raw("  "), Span::styled("Hex: ", Style::default().fg(LAVNDR)), Span::styled("48 65 6C  or  48,65,6C",     Style::default().fg(TEXT))]),
        Line::from(""),
        sec("Other"),
        row("F2",            "ASCII reference table"),
        row("F1",            "This help screen"),
        row("Esc / Ctrl+C",  "Quit"),
        Line::from(""),
        Line::from(vec![Span::raw("  "), Span::styled("Press Esc to close", Style::default().fg(TEXT_DIM))]),
    ];

    f.render_widget(Paragraph::new(lines), inner);
}