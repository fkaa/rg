use crate::{
    Context, MouseButton,
    Background,
    TextStyle,
    Border,
    TextAlignment,
    ButtonFlags,
    WindowFlags,
    RowType,
    Id,
    PanelFlags,
    PoolIndex,
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
    tabs: Vec<TabItem>,

    want_layout: bool,
    prev_frame_visible: u32,
    curr_frame_visible: u32,
    last_tab_item_index: usize,
    selected_tab_id: Id,
    visible_tab_id: Id,
    next_selected_tab: Id,
    offset_next_tab: f32,
    offset_max: f32,
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
            bounds: Rect::new(float2(0f32, 0f32), float2(0f32, 0f32)),
            tabs: Vec::new(),

            want_layout: false,
            prev_frame_visible: 0,
            curr_frame_visible: 0,
            last_tab_item_index: 0,
            selected_tab_id: 0,
            visible_tab_id: 0,
            next_selected_tab: 0,
            offset_next_tab: 0f32,
            offset_max: 0f32,
        }
    }
}

#[derive(Copy, Clone)]
pub struct TabItemSort {
    index: usize,
    width: f32,
}

#[derive(Clone)]
pub struct TabItem {
    id: Id,
    content_width: f32,
    width: f32,
    offset: f32,
    name: String,

    last_frame_visible: u32,
}

impl TabItem {
    pub fn new() -> Self {
        TabItem {
            id: 0,
            content_width: 0f32,
            width: 0f32,
            offset: 0f32,
            name: String::new(),

            last_frame_visible: 0,
        }
    }
}



impl Context {
    pub fn begin_tab_bar(&mut self, title: &str) -> bool {
        let hash = hash_id(title);
        let tab_idx = self.tab_bars.get(hash);
        
        let padding = self.style.tab.padding;
        let tab_height = 20f32 + padding.1 * 2f32;

        self.row(RowType::dynamic(1));
        self.column(Some(1f32));
        let width = self.panel_peek_width();
        let origin = self.panel_cursor();
        //let bounds = Rect::new(origin, origin + float2(width, tab_height));
        let (bounds, _state) = self.widget(Some(tab_height));

        let tab = &mut self.tab_bars[tab_idx];
        tab.id = hash;
        self.current_tab_bar.push(tab_idx);
        tab.bounds = bounds;
        tab.want_layout = true;
        tab.prev_frame_visible = tab.curr_frame_visible;
        tab.curr_frame_visible = self.frame;

        //self.draw_list.add_line(float2(bounds.min.0, bounds.max.1), bounds.max, 0xffffffff);

        self.row(RowType::fixed());
        
        true
    }

    pub fn end_tab_bar(&mut self) {
        if let Some(idx) = self.current_tab_bar.last() {
            let tab = &mut self.tab_bars[*idx];
            if tab.want_layout {
                self.tab_bar_layout(*idx);
            }
        }
        
        self.current_tab_bar.pop();
    }

    pub fn begin_tab_item(&mut self, title: &str) -> bool {
        let id = self.id(title);
        let width = self.default_font.text_width(&mut *self.renderer, title);
        let padding = self.style.tab.padding;
        self.last_widget_state = WidgetState::None;

        let tab_bar_idx = *self.current_tab_bar.last()
            .expect("Needs to be called between begin_tab_bar() and end_tab_bar()");
        let tab_bar = &self.tab_bars[tab_bar_idx]; // borrowck
        if tab_bar.want_layout {
            self.tab_bar_layout(tab_bar_idx);
        }

        let tab_bar = &mut self.tab_bars[tab_bar_idx]; // borrowck
        if tab_bar.selected_tab_id == 0 {
            tab_bar.selected_tab_id = id;
        }
        let tab_idx = self.find_tab(tab_bar_idx, id);
        let tab_bar = &mut self.tab_bars[tab_bar_idx];

        let (is_new, tab_idx) = if let Some(idx) = tab_idx {
            (false, idx)
        } else {
            let mut item = TabItem::new();
            item.id = id;
            item.width = width + padding.0 * 2f32;;
            item.name += title;
            let idx = tab_bar.tabs.len();
            
            tab_bar.tabs.push(item);
            
            (true, idx)
        };

        let padding = self.style.tab.padding;
        let tab = &mut tab_bar.tabs[tab_idx];
        tab.last_frame_visible = self.frame;

        /*tab.offset = tab_bar.offset_next_tab;
        tab_bar.offset_next_tab = tab.width + padding.0 * 2f32;*/
        
        let width = tab.width;
        let offset = tab.offset;

        let tab_height = 20f32 + padding.1 * 2f32;
        //self.column_fixed(width);
        //let (bounds, state) = self.widget(Some(tab_height));
        let origin = tab_bar.bounds.min + float2(offset, 0f32);
        let bounds = Rect::new(origin, origin + float2(width, tab_height));
 
        drop(tab_bar); // borrowck

        let pressed = self.button_behaviour(bounds, ButtonFlags::PressOnClick);
        
        let index = self.current_index();
        let wnd = unsafe { self.windows.get_unchecked(index) };
        let tab_bar = &mut self.tab_bars[tab_bar_idx];

        if pressed && !wnd.flags.contains(WindowFlags::ReadOnly) {
            tab_bar.next_selected_tab = id;
        }
        
        let style = &self.style.tab;
        let state = self.last_widget_state;
        let is_current = tab_bar.selected_tab_id == id;
        let (text, border, background) = if is_current {
            (style.active_text, style.active_border, style.active)
        } else if state.contains(WidgetState::Hovering) {
            (style.hover_text, style.hover_border, style.hover)
        } else {
            (style.normal_text, style.normal_border, style.normal)
        };
        self.tab_item_bg(bounds, background, border);
        self.tab_item_text(title, bounds, text);
        
        if self.io.is_mouse_down(MouseButton::Right) {
            self.draw_list.add_rect(bounds.min, bounds.max, 0f32, 1f32, 0x8800ffff);
        }

        if is_current {
            let height = self.available_height();
            self.row(RowType::dynamic_ex(1, height));
            self.column(Some(1f32));
            let (bounds, _state) = self.widget(None);
            self.begin_panel_ex(title, Some(bounds), Some((offset, width)), PanelFlags::Styled);
        }
        
        is_current
    }

    pub fn end_tab_item(&mut self) {
        self.end_panel();
    }

    fn tab_bar_layout(&mut self, tab_bar_idx: PoolIndex) {
        let tab_bar = &mut self.tab_bars[tab_bar_idx];
        tab_bar.want_layout = false;
        
        // garbage collect
        let mut tab_dst_n = 0;
        let mut i = 0;
        let len = tab_bar.tabs.len();
        while i < len {
            let tab = &tab_bar.tabs[i];
            
            if tab.last_frame_visible < tab_bar.prev_frame_visible {
                if tab.id == tab_bar.selected_id {
                    tab_bar.selected_id = 0;
                    continue;
                }
            }

            if tab_dst_n != i {
                tab_bar.tabs.swap(tab_dst_n, i);
                // tab_bar.tabs[tab_dst_n] = *tab;
            }
            
            tab_dst_n += 1;
            i += 1;
        }

        if tab_bar.tabs.len() != tab_dst_n {
            tab_bar.tabs.truncate(tab_dst_n);
        }

        // commit tab selection
        if tab_bar.next_selected_tab != 0 {
            tab_bar.selected_tab_id = tab_bar.next_selected_tab;
            tab_bar.next_selected_tab = 0;
        }

        // ideal width, figure out if we should resize tab width
        // and/or do clipping
        let mut total_width = 0f32;
        let mut found = false;
        
        let mut i = 0;
        let len = tab_bar.tabs.len();
        self.tab_sorter.resize(len, TabItemSort {
            index: 0,
            width: 0f32
        });
        
        while i < len {
            let tab = &mut tab_bar.tabs[i];

            if tab.id == tab_bar.selected_id {
                found = true;
            }

            let name = &tab.name;

            let padding = self.style.tab.padding;
            tab.content_width = self.default_font.text_width(&mut *self.renderer, name) + padding.0 * 2f32;
            total_width += tab.content_width + 2f32;

            self.tab_sorter[i] = TabItemSort {
                index: i,
                width: tab.content_width,
            };
            
            i += 1;
        }

        let width_available = tab_bar.bounds.width();
        let mut excess = if width_available < total_width {
            total_width - width_available
        } else {
            0f32
        };
        let tabs_len = tab_bar.tabs.len();

        // if we overflow, shrink as much as possible
        if excess > 0f32 {
            if tab_bar.tabs.len() > 1 {
                self.tab_sorter.sort_by(
                    |a, b| {
                        a.width.partial_cmp(&b.width).unwrap()
                            .then(a.index.cmp(&b.index))
                    }
                );

                let mut tab_count_same_width = 1;
                while excess > 0f32 && tab_count_same_width < tabs_len {
                    let mut first_width = self.tab_sorter[0].width;
                    let mut curr_width = self.tab_sorter[tab_count_same_width].width;
                    while tab_count_same_width < tabs_len && first_width == curr_width {
                        tab_count_same_width += 1;
                        if tab_count_same_width == tabs_len { break; }
                        curr_width = self.tab_sorter[tab_count_same_width].width;
                    }

                    let max_remove = if tab_count_same_width < tabs_len {
                        first_width - curr_width
                    } else {
                        first_width - 1f32
                    };
                    let remove = (excess / tab_count_same_width as f32).min(max_remove);

                    let mut i = 0;
                    while i < tab_count_same_width {
                        self.tab_sorter[i].width -= remove;
                        i += 1;
                    }

                    excess -= remove * tab_count_same_width as f32;
                }

                let mut i = 0;
                while i < tabs_len {
                    let sorted_tab = &self.tab_sorter[i];
                    
                    tab_bar.tabs[sorted_tab.index].width = sorted_tab.width.floor();
                    i += 1;
                }
            }
        } else {
            let max_width = 20f32 * 20f32;
            let mut i = 0;
            while i < tabs_len {
                let tab_bar = &mut tab_bar.tabs[i];
                tab_bar.width = tab_bar.content_width.min(max_width);
                i += 1;
            }
        }

        let spacing_x = 2f32;
        let mut offset_x = 0f32;
        let mut i = 0;
        while i < tabs_len {
            let tab = &mut tab_bar.tabs[i];
            tab.offset = offset_x;
            offset_x += tab.width + spacing_x;
            i += 1;
        }

        tab_bar.offset_max = (offset_x - spacing_x).max(0f32);
        tab_bar.offset_next_tab = 0f32;

        if !found {
            tab_bar.selected_id = 0;
        }
    }
    
    fn find_tab(&self, tab_bar_idx: PoolIndex, id: Id) -> Option<usize> {
        let tab_bar = &self.tab_bars[tab_bar_idx];
        
        let mut i = 0;
        let len = tab_bar.tabs.len();
        
        while i < len {
            let tab = unsafe { tab_bar.tabs.get_unchecked(i) };
            if tab.id == id {
                return Some(i);
            }
            
            i += 1;
        }
        
        None
    }

    fn tab_item_bg(&mut self, bounds: Rect, bg: Background, border: Border) {
        let rounding = border.rounding;

        let bmin = bounds.min.round();
        let bmax = bounds.max.round();
        
        let y1 = bmin.1 + 1f32;
        let y2 = bmax.1 - 1f32;
        
        match bg {
            Background::Color(c) => {
                self.draw_list.path()
                    .line(float2(bmin.0, y2))
                    .arc_fast(float2(bmin.0 + rounding, y1 + rounding), rounding, 6, 9)
                    .arc_fast(float2(bmax.0 - rounding, y1 + rounding), rounding, 9, 12)
                    .line(float2(bmax.0, y2))
                    .fill(c);
            }
            _ => {}
        }

        self.draw_list.path()
            .line(float2(bmin.0, y2))
            .arc_fast(float2(bmin.0 + rounding, y1 + rounding), rounding, 6, 9)
            .arc_fast(float2(bmax.0 - rounding, y1 + rounding), rounding, 9, 12)
            .line(float2(bmax.0, y2))
            .stroke_gradient(border.thickness, false, border.color & 0x00ffffff, border.color);
    }

    fn tab_item_text(&mut self, text: &str, bounds: Rect, style: TextStyle) {
        let yoff = (bounds.height() - self.default_font.advance_y()) * 0.5f32;
        self.draw_list.add_text_wrapped(
            &mut *self.renderer,
            &mut self.default_font,
            text,
            bounds.min + float2(0f32, yoff),
            style.align,
            bounds.width(),
            style.color);
    }
}
