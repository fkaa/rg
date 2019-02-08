use crate::{
    Context, MouseButton,
    Background,
    TextStyle,
    Border,
    TextAlignment,
    ParagraphStyle,
    
    math::{
        float2,
        Rect,
    }
};

use super::{
    WidgetState,
};

impl Context {
    fn paragraph_height(&mut self, text: &str, wrap: f32) -> f32 {
        let style = &self.style.paragraph;
        let h = self.default_font.text_size(&mut *self.renderer, text, wrap).1;

        h + style.padding.1 * 2f32
    }
    
    pub fn paragraph_styled(&mut self, text: &str, bounds: Rect, padding: float2, style: TextStyle) {
        self.draw_list.add_text_wrapped(
            &mut *self.renderer,
            &mut self.default_font,
            text,
            bounds.min,
            style.align,
            bounds.width(),
            style.color
        );
    }
    
    pub fn paragraph(&mut self, text: &str) {
        self.last_widget_state = WidgetState::None;

        let w = self.peek_widget_width();
        let h = self.paragraph_height(text, w);

        let (bounds, state) = self.widget(Some(h));

        let style = &self.style.paragraph;
        self.paragraph_styled(text, bounds, style.padding, style.normal_text);
    }
}
