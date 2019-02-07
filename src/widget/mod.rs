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

pub use self::button::*;
pub use self::window::*;

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

pub enum TextAlignment {
    Left,
    Centered,
    Right,
}

pub enum Background {
    NinePatch(TextureHandle, [float2; 4]),
    Texture(TextureHandle),
    Color(u32),
}

impl Context {
    pub fn widget(&mut self) -> (Rect, WidgetLayoutState) {
        let bounds = self.panel_alloc_space();

        let index = self.current_index();
        let mut wnd = unsafe { self.windows.get_unchecked_mut(index) };

        let clip = wnd.layout.clip;
        
        (bounds, WidgetLayoutState::Visible)
    }
}
