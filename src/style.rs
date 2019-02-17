use crate::{
    draw::TextureHandle,
    
    math::{*}
};

pub type UColor = u32;

#[inline(always)]
const fn make_color(r: u8, g: u8, b: u8, a: u8) -> UColor {
    (a as u32) << 24 |
    (b as u32) << 16 |
    (g as u32) << 8 |
    (r as u32)
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum CursorType {
    Default,
    Caret,
    ResizeVertical,
    ResizeHorizontal,
    ResizeNE,
    ResizeNW,
}

#[derive(Copy, Clone)]
pub enum TextAlignment {
    Left,
    Centered,
    Right,
}

#[derive(Copy, Clone)]
pub struct FontFace(usize);

impl FontFace {
    pub fn default_font() -> Self {
        FontFace(0)
    }
}

#[derive(Copy, Clone)]
pub struct TextStyle {
    pub face: FontFace,
    pub align: TextAlignment,
    pub color: UColor,
}

impl TextStyle {
    pub fn default_style() -> Self {
        TextStyle {
            face: FontFace::default_font(),
            align: TextAlignment::Left,
            color: make_color(0, 0, 0, 255),
        }
    }
}

#[derive(Copy, Clone)]
pub struct Border {
    pub color: UColor,
    pub rounding: f32,
    pub thickness: f32,
}

impl Border {
    pub fn new(color: UColor, thickness: f32, rounding: f32) -> Self {
        Border {
            color,
            thickness,
            rounding
        }
    }
    
    pub fn default_border() -> Self {
        Border {
            color: 0,
            rounding: 0f32,
            thickness: 0f32,
        }
    }
}

#[derive(Copy, Clone)]
pub enum Background {
    NinePatch(TextureHandle, [float2; 4]),
    Texture(TextureHandle),
    Color(UColor),
}

pub struct WindowHeaderStyle {
    pub spacing: float2,
    pub padding: float2,
    pub label_padding: float2,

    pub normal_text: TextStyle,
    pub hover_text: TextStyle,
    pub active_text: TextStyle,

    pub normal: Background,
    pub hover: Background,
    pub active: Background,
}

impl WindowHeaderStyle {
    pub fn new() -> Self {
        WindowHeaderStyle {
            spacing: float2(0f32, 0f32),
            padding: float2(4f32, 4f32),
            label_padding: float2(2f32, 2f32),

            normal_text: TextStyle::default_style(),
            hover_text: TextStyle::default_style(),
            active_text: TextStyle::default_style(),

            normal: Background::Color(0xff000000),
            hover: Background::Color(0xff000000),
            active: Background::Color(0xff000000),
        }
    }
}

pub struct WindowStyle {
    pub header: WindowHeaderStyle,
    pub spacing: float2,
    pub padding: float2,
    pub panel_padding: float2,
    pub scrollbar_size: float2,

    pub normal_border: Border,
    pub hover_border: Border,
    pub active_border: Border,
    
    pub normal: Background,
    pub hover: Background,
    pub active: Background,
}

impl WindowStyle {
    pub fn new() -> Self {
        WindowStyle {
            header: WindowHeaderStyle::new(),
            spacing: float2(0f32, 0f32),
            padding: float2(0f32, 0f32),
            panel_padding: float2(4f32, 4f32),
            scrollbar_size: float2(10f32, 10f32),
            
            normal_border: Border::default_border(),
            hover_border: Border::default_border(),
            active_border: Border::default_border(),
            
            normal: Background::Color(0),
            hover: Background::Color(0),
            active: Background::Color(0),
        }
    }
}

pub struct ButtonStyle {
    pub padding: float2,

    pub normal_text: TextStyle,
    pub hover_text: TextStyle,
    pub active_text: TextStyle,

    pub normal_border: Border,
    pub hover_border: Border,
    pub active_border: Border,
    
    pub normal: Background,
    pub hover: Background,
    pub active: Background,
}

impl ButtonStyle {
    pub fn new() -> Self {
        ButtonStyle {
            padding: float2(0f32, 0f32),

            normal_text: TextStyle::default_style(),
            hover_text: TextStyle::default_style(),
            active_text: TextStyle::default_style(),

            normal_border: Border::default_border(),
            hover_border: Border::default_border(),
            active_border: Border::default_border(),
            
            normal: Background::Color(0),
            hover: Background::Color(0),
            active: Background::Color(0),
        }
    }
}

pub struct TabStyle {
    pub padding: float2,

    pub normal_text: TextStyle,
    pub hover_text: TextStyle,
    pub active_text: TextStyle,

    pub normal_border: Border,
    pub hover_border: Border,
    pub active_border: Border,
    
    pub normal: Background,
    pub hover: Background,
    pub active: Background,
}

impl TabStyle {
    pub fn new() -> Self {
        TabStyle {
            padding: float2(0f32, 0f32),

            normal_text: TextStyle::default_style(),
            hover_text: TextStyle::default_style(),
            active_text: TextStyle::default_style(),

            normal_border: Border::default_border(),
            hover_border: Border::default_border(),
            active_border: Border::default_border(),
            
            normal: Background::Color(0),
            hover: Background::Color(0),
            active: Background::Color(0),
        }
    }
}

pub struct ParagraphStyle {
    pub padding: float2,

    pub normal_text: TextStyle,
}

impl ParagraphStyle {
    pub fn new() -> Self {
        ParagraphStyle {
            padding: float2(0f32, 0f32),

            normal_text: TextStyle::default_style(),
        }
    }
}

pub struct Style {
    pub window: WindowStyle,
    pub button: ButtonStyle,
    pub tab: TabStyle,
    pub paragraph: ParagraphStyle,
}

const ACTIVE_DARK_BTN_BG: UColor = make_color(39, 42, 44, 255);
const HOVER_DARK_BTN_BG: UColor = make_color(93, 96, 104, 255);
const NORMAL_DARK_BTN_BG: UColor = make_color(76, 80, 86, 255);

const ACTIVE_DARK_BTN_TEXT: UColor = make_color(230, 230, 230, 255);
const HOVER_DARK_BTN_TEXT: UColor = make_color(240, 240, 240, 255);
const NORMAL_DARK_BTN_TEXT: UColor = make_color(220, 220, 220, 255);


const NORMAL_DARK_BG: UColor = make_color(60, 59, 64, 255);
const NORMAL_DARK_BG_TINT: UColor = make_color(76, 79, 79, 255);

const NORMAL_DARK_TEXT: UColor = make_color(242, 241, 239, 255);
const NORMAL_DARK_TEXT_FADE: UColor = make_color(189, 195, 199, 255);

impl Style {
    pub fn new() -> Self {
        Style {
            window: WindowStyle::new(),
            button: ButtonStyle::new(),
            tab: TabStyle::new(),
            paragraph: ParagraphStyle::new(),
        }
    }

    fn dark_window() -> WindowStyle {
        let mut header = WindowHeaderStyle::new();
        header.hover_text.color = NORMAL_DARK_TEXT_FADE;
        header.active_text.color = NORMAL_DARK_TEXT_FADE;
        header.normal_text.color = NORMAL_DARK_TEXT_FADE;
        
        let header_bg = Background::Color(NORMAL_DARK_BG_TINT);
        header.normal = header_bg;
        header.hover = header_bg;
        header.active = header_bg;

        let mut style = WindowStyle::new();

        style.header = header;

        style.padding = float2(2f32, 2f32);
        style.spacing = float2(2f32, 0f32);

        let window_bg = Background::Color(NORMAL_DARK_BG);
        style.normal = window_bg;
        style.hover = window_bg;
        style.active = window_bg;

        let window_border = Border::new(make_color(30, 30, 30, 255), 2f32, 0f32);
        style.normal_border = window_border;
        style.hover_border = window_border;
        style.active_border = window_border;
        
        style
    }

    pub fn dark_button() -> ButtonStyle {
        let mut style = ButtonStyle::new();

        style.active = Background::Color(ACTIVE_DARK_BTN_BG);
        style.hover = Background::Color(HOVER_DARK_BTN_BG);
        style.normal = Background::Color(NORMAL_DARK_BTN_BG);

        style.active_border = Border::new(make_color(10, 10, 10, 255), 1f32, 0f32);
        style.hover_border = Border::new(make_color(30, 30, 30, 255), 1f32, 0f32);
        style.normal_border = Border::new(make_color(20, 20, 20, 255), 1f32, 0f32);

        style.active_text.color = ACTIVE_DARK_BTN_TEXT;
        style.hover_text.color = HOVER_DARK_BTN_TEXT;
        style.normal_text.color = NORMAL_DARK_BTN_TEXT;

        style.active_text.align = TextAlignment::Centered;
        style.hover_text.align = TextAlignment::Centered;
        style.normal_text.align = TextAlignment::Centered;
        
        style
    }

    pub fn dark_tabs() -> TabStyle {
        let mut style = TabStyle::new();

        style.padding = float2(8f32, 4f32);
        
        style.active = Background::Color(NORMAL_DARK_BG);
        style.hover = Background::Color(HOVER_DARK_BTN_BG);
        style.normal = Background::Color(ACTIVE_DARK_BTN_BG);

        style.active_border = Border::new(make_color(30, 30, 30, 255), 2f32, 0f32);
        style.hover_border = Border::new(make_color(10, 10, 10, 0), 2f32, 0f32);
        style.normal_border = Border::new(make_color(20, 20, 20, 0), 2f32, 0f32);

        style.active_text.color = ACTIVE_DARK_BTN_TEXT;
        style.hover_text.color = HOVER_DARK_BTN_TEXT;
        style.normal_text.color = make_color(150, 150, 150, 255);

        style.active_text.align = TextAlignment::Centered;
        style.hover_text.align = TextAlignment::Centered;
        style.normal_text.align = TextAlignment::Centered;
        
        style
    }

    pub fn dark_paragraph() -> ParagraphStyle {
        let mut style = ParagraphStyle::new();

        style.normal_text.color = NORMAL_DARK_TEXT;
        style.normal_text.align = TextAlignment::Centered;
        
        style
    }
    
    pub fn dark_style() -> Self {
        let mut style = Style::new();

        style.window = Self::dark_window();
        style.button = Self::dark_button();
        style.tab = Self::dark_tabs();
        style.paragraph = Self::dark_paragraph();

        style
    }
}
