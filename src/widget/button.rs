use crate::{
    Context, MouseButton,
    
    math::{
        float2,
        Rect,
    }
};

use super::{
    WidgetState,
    Background,
    TextAlignment,
};

pub struct ButtonStyle {
    padding: float2,
    background: Background,
    text_align: TextAlignment,
}

impl ButtonStyle {
    pub fn new() -> Self {
        ButtonStyle {
            padding: float2(0f32, 0f32),
            background: Background::Color(0xffffffff),
            text_align: TextAlignment::Centered,
        }
    }
}

impl Context {
    pub fn button(&mut self) -> bool {
        //let pressed = self.button_behaviour();
        true
    }

    pub fn do_button(&mut self, bounds: Rect) -> (Rect, bool) {
        let style = &self.style.button;
        
        let min = bounds.min + style.padding;
        let max = bounds.max - style.padding;

        (Rect::new(min, max), self.button_behaviour(bounds))
    }

    pub fn draw_button(&mut self, bounds: Rect) {
        let color = 0xffffff00;

        self.draw_list.add_rect_filled(bounds.min,bounds.max, 0f32, color);
    }
    
    pub fn draw_button_text(&mut self, bounds: Rect, content: Rect, text: &str) {
        let color = 0xffffffff;
        
        self.draw_button(bounds);

        let style = &self.style.button;
        let w = self.default_font.text_width(&mut *self.renderer, text);
        let x_offset = match style.text_align {
            TextAlignment::Left => 0f32,
            TextAlignment::Right => bounds.width() - w,
            TextAlignment::Centered => bounds.width() * 0.5f32 - w * 0.5f32,
        };
        let y_offset = self.default_font.height();

        self.draw_list.add_text(&mut *self.renderer, &mut self.default_font, text, float2(x_offset, y_offset), color);
    }
    
    pub fn do_button_text(&mut self, bounds: Rect, text: &str) -> bool {
        let (content, pressed) = self.do_button(bounds);
        
        self.draw_button_text(bounds, content, text);

        pressed
    }
    
    pub fn button_text(&mut self, text: &str) -> bool {
        let (bounds, state) = self.widget();

        // dbg!(bounds);

        self.do_button_text(bounds, text)
    }
    
    pub fn button_behaviour(&mut self, bounds: Rect) -> bool {
        let io = &self.io;
        let mut pressed = false;
        let state = &mut self.last_widget_state;

        if io.has_mouse_in_rect(MouseButton::Left, bounds) {
            *state = WidgetState::Hovering;

            if io.is_mouse_down(MouseButton::Left) {
                *state = WidgetState::Active;
            }

            if io.has_mouse_click_in_rect(MouseButton::Left, bounds) {
                if io.is_mouse_released(MouseButton::Left) {
                    pressed = true;
                }
            }
        }

        pressed
    }
}
