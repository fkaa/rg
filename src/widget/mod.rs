//use bitflags;
use crate::{
    Context,
    
    draw::{
        TextureHandle,
    },
    math::{
        float2,
        Rect,
    },
};

mod button;
mod window;
mod text;
mod textfield;
mod tab;
mod panel;
mod slider;

pub use self::button::*;
pub use self::window::*;
pub use self::text::*;
pub use self::textfield::*;
pub use self::tab::*;
pub use self::panel::*;

bitflags! {
    pub struct WidgetState: u32 {
        const None = 0;
        const Modified = 1 << 0;
        const Inactive = 1 << 1;
        const Entered = 1 << 2;
        const Hovering = 1 << 3;
        const Activated = 1 << 4;
        const Left = 1 << 5;
        
        const Hovered = Self::Hovering.bits | Self::Modified.bits;
        const Active = Self::Activated.bits | Self::Modified.bits;
    }
}

pub enum WidgetLayoutState {
    Hidden,
    Visible,
    PartiallyVisible,
}

impl Context {
    pub fn peek_widget_width(&mut self) -> f32 {
        self.panel_peek_width()
    }
    
    pub fn widget(&mut self, height: Option<f32>) -> (Rect, WidgetLayoutState) {
        let bounds = self.panel_alloc_space(height, true);
        
        (bounds, WidgetLayoutState::Visible)
    }

    pub fn widget_no_style(&mut self, height: Option<f32>) -> (Rect, WidgetLayoutState) {
        let bounds = self.panel_alloc_space(height, false);
        
        (bounds, WidgetLayoutState::Visible)
    }
}
