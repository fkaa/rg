use crate::{
    Context, MouseButton,
    Background,
    TextStyle,
    Border,
    TextAlignment,
    
    math::{
        float2,
        Rect,
    }
};

use super::{
    WidgetState,
};

bitflags!{
    pub struct ButtonFlags: u32 {
        const None = 0;
        const PressOnClick = 1 << 0;
    }
}

impl Context {
    fn button_width(&mut self, text: &str) -> f32 {
        let style = &self.style.button;
        let w = self.default_font.text_width(&mut *self.renderer, text);

        w + style.padding.0 * 2f32
    }
    
    fn button_height(&mut self, text: &str, wrap: f32) -> f32 {
        let style = &self.style.button;
        let h = self.default_font.text_size(&mut *self.renderer, text, wrap).1;

        h + style.padding.1 * 2f32
    }

    pub fn do_button(&mut self, bounds: Rect) -> (Rect, bool) {
        let style = &self.style.button;
        
        let min = bounds.min + style.padding;
        let max = bounds.max - style.padding;

        (Rect::new(min, max), self.button_behaviour(bounds, ButtonFlags::None))
    }

    pub fn draw_button(&mut self, bounds: Rect, border: Border, background: Background) {
        match background {
            Background::Color(col) => {
                let inner = bounds.pad(1f32);
                self.draw_list.add_rect_filled(inner.min, inner.max, border.rounding, col);
                self.draw_list.add_rect(bounds.min, bounds.max, border.rounding, border.thickness, border.color);
            },
            _ => {}
        }
    }
    
    pub fn draw_button_text(&mut self, bounds: Rect, content: Rect, text: TextStyle, border: Border, background: Background, title: &str) {        
        self.draw_button(bounds, border, background);

        self.draw_list.add_text_wrapped(&mut *self.renderer, &mut self.default_font, title, bounds.min + float2(0f32, 0f32), text.align, bounds.width(), text.color);
    }
    
    pub fn do_button_text(&mut self, bounds: Rect, title: &str) -> bool {
        let (content, pressed) = self.do_button(bounds);

        let style = &self.style.button;
        let state = self.last_widget_state;

        let (text, border, background) = if state.contains(WidgetState::Active) {
            (style.active_text, style.active_border, style.active)
        } else if state.contains(WidgetState::Hovering) {
            (style.hover_text, style.hover_border, style.hover)
        } else {
            (style.normal_text, style.normal_border, style.normal)
        };
        
        self.draw_button_text(bounds, content, text, border, background, title);

        pressed
    }
    
    pub fn button_text(&mut self, text: &str) -> bool {
        // TODO: maybe make wrapping optional
        self.last_widget_state = WidgetState::None;
        
        let w = self.peek_widget_width();
        let h = self.button_height(text, w);
           
        let (bounds, state) = self.widget(Some(h));

        // dbg!(bounds);

        self.do_button_text(bounds, text)
    }
    
    pub fn button_behaviour(&mut self, bounds: Rect, flags: ButtonFlags) -> bool {
        let io = &self.io;
        let mut pressed = false;
        let state = &mut self.last_widget_state;

        let mouse_over_button = io.has_mouse_in_rect(bounds);
        let mouse_click_rect = io.has_mouse_click_in_rect(MouseButton::Left, bounds);
        
        if io.has_mouse_in_rect(bounds) {
            *state = WidgetState::Hovering;

            if io.is_mouse_down(MouseButton::Left) {
                *state = WidgetState::Active;
            }

            if io.has_mouse_click_in_rect(MouseButton::Left, bounds) {
                if flags.contains(ButtonFlags::PressOnClick) {
                    if io.is_mouse_pressed(MouseButton::Left) {
                        pressed = true;
                    }
                } else {
                    if io.is_mouse_released(MouseButton::Left) {
                        pressed = true;
                    }
                }
            }
        }

        pressed
    }
}
