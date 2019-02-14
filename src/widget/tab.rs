use crate::{
    Context, MouseButton,
    Background,
    TextStyle,
    Border,
    TextAlignment,
    Id,
    hash_id,
    
    math::{
        float2,
        Rect,
    }
};

use super::{
    WidgetState,
};

#[derive(Clone)]
pub struct TabBar {
    id: Id,
    selected_id: Id,
    bounds: Rect,
}

impl Default for TabBar {
    fn default() -> Self {
        TabBar::new()
    }
}

impl TabBar {
    pub fn new() -> Self {
        TabBar {
            id: 0,
            selected_id: 0,
            bounds: Rect::new(float2(0f32, 0f32), float2(0f32, 0f32))
        }
    }
}

pub struct TabItem {
    id: Id,
    
}


impl Context {
    pub fn begin_tab_bar(&mut self, title: &str) -> bool {
        let hash = hash_id(title);
        let tab_idx = self.tab_bars.get(hash);

        let tab_height = 20f32;
        let (bounds, _state) = self.widget(Some(tab_height));

        let tab = &mut self.tab_bars[tab_idx];
        tab.id = hash;
        self.current_tab_bar.push(tab_idx);
        tab.bounds = bounds;

        self.draw_list.add_line(float2(bounds.min.0, bounds.max.1), bounds.max, 0xffffffff);
        
        false
    }

    pub fn end_tab_bar(&mut self) {
        if let Some(idx) = self.current_tab_bar.last() {
            let tab = &mut self.tab_bars[*idx];
        }
        
        self.current_tab_bar.pop();
    }
}
