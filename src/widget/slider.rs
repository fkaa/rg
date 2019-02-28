use crate::{
    Context, MouseButton,
    Background,
    TextStyle,
    Border,
    TextAlignment,
    Id,
    DataType,
    
    math::{
        float2,
        Rect,
    }
};

use super::{
    WidgetState,
};

pub enum SliderData {
    F32(*mut f32, f32, f32),
}

impl Context {
    fn slider_behaviour_f(&mut self, bounds: Rect, id: Id, ptr: *mut f32, min: f32, max: f32) -> (Rect, bool) {
        unimplemented!()
    }
    
    fn slider_behaviour(&mut self, bounds: Rect, id: Id, ty: SliderData) -> (Rect, bool) {
        match ty {
            SliderData::F32(ptr, min, max) => {
                self.slider_behaviour_f(bounds, id, ptr, min, max)
            }
            _ => {
                unimplemented!()
            }
        }
    }
    
    pub fn sliderf_tt(&mut self, label: &str, tt: &mut tt::Tt, prop: tt::StringId) {
        let h = self.default_font.height();
        let (bounds, state) = self.widget(Some(h));
    }
}
