use crate::{
    Context, Window, WindowStyle, WindowFlags, MouseButton,
    
    math::{*},
};

mod row;

pub use selfrow::*;

pub enum RowLayoutType {
    DynamicFixed,
    DynamicRow,
    DynamicFree,
    Dynamic,
    StaticFixed,
    StaticRow,
    StaticFree,
    Static,
}



pub struct RowLayout {
    ty: RowLayoutType,
    index: u32,
    height: f32,
    min_height: f32,

    max_x: f32,
    x: f32,
    y: f32,

    ratio: f32,
    columns: u32,
    item_width: f32,
    item_height: f32,
    item_offset: f32,
    filled: f32,
    item: Rect,
}

impl RowLayout {
    pub fn new() -> Self {
        RowLayout {
            ty: RowLayoutType::StaticFixed,
            index: 0,
            height: 0f32,
            min_height: 0f32,
            max_x: 0f32,
            x: 0f32,
            y: 0f32,
            ratio: 0f32,
            columns: 0,
            item_width: 0f32,
            item_height: 0f32,
            item_offset: 0f32,
            filled: 0f32,
            item: Rect::new(float2(0f32, 0f32), float2(0f32, 0f32)),
        }
    }
}

pub enum PanelType {
    Window,
    Group,
    Popup,
    Contextual,
    Combo,
    Menu,
    Tooltip,
}

pub struct Panel {
    ty: PanelType,
    bounds: Rect,
    row: RowLayout,
    flags: WindowFlags,
    
    cursor: float2,
    header_height: f32,
    footer_height: f32,
    max_x: f32,
    has_scrolling: bool,
    
    pub clip: Rect,
    pub offset: float2,
}

impl Panel {
    pub fn new() -> Self {
        Panel {
            ty: PanelType::Window,
            bounds: Rect::new(float2(0f32, 0f32), float2(0f32, 0f32)),
            row: RowLayout::new(),
            flags: WindowFlags::None,
            cursor: float2(0f32, 0f32),
            header_height: 0f32,
            footer_height: 0f32,
            max_x: 0f32,
            has_scrolling: true,
            clip: Rect::new(float2(0f32, 0f32), float2(0f32, 0f32)),
            offset: float2(0f32, 0f32),
        }
    }

    pub fn layout(&mut self, height: f32, columns: u32, item_spacing: float2) {
        self.row.index = 0;
        self.cursor.1 += self.row.height;
        self.row.columns = columns;
        
        if height == 0f32 {
            self.row.height = self.row.min_height + item_spacing.1;
        } else {
            self.row.height = height + item_spacing.1;
        }

        self.row.item_offset = 0f32;
    }
    
    pub fn get_padding(&self, style: &WindowStyle) -> float2 {
        match self.ty {
            PanelType::Window => style.padding,
            PanelType::Group => style.padding,
            PanelType::Popup => style.padding,
            PanelType::Contextual => style.padding,
            PanelType::Combo => style.padding,
            PanelType::Menu => style.padding,
            PanelType::Tooltip => style.padding,
        }
    }

    pub fn calculate_usable_space(&self, style: &WindowStyle) -> f32 {
        let spacing = style.spacing;
        let padding = self.get_padding(style);

        let mut columns = self.row.columns;
        if columns > 0 {
            columns -= 1;
        }

        let panel_padding = 2f32 * padding.0;
        let panel_spacing = columns as f32 * spacing.0;

        self.bounds.width() - panel_padding - panel_spacing
    }
}

impl Context {
    pub fn panel_alloc_row(&mut self) {
        let index = self.current_index();
        let mut wnd = unsafe { self.windows.get_unchecked_mut(index) };
        
        let spacing = float2(5f32, 5f32);
        let row_height = wnd.layout.row.height - spacing.1;
        
        wnd.layout.layout(row_height, wnd.layout.row.columns, spacing);
    }
    
    pub fn panel_alloc_space(&mut self) -> Rect {
        let index = self.current_index();
        let (i, c) = {
            let mut wnd = unsafe { self.windows.get_unchecked_mut(index) };

            (wnd.layout.row.index, wnd.layout.row.columns)
        };
        if i >= c {
            self.panel_alloc_row();
        }

        let bounds = self.layout_widget_space(true);
        let mut wnd = unsafe { self.windows.get_unchecked_mut(index) };
        
        wnd.layout.row.index += 1;

        bounds
    }

    pub fn layout_widget_space(&mut self, modify: bool) -> Rect {
        let index = self.current_index();
        let mut wnd = unsafe { self.windows.get_unchecked_mut(index) };

        let style = &self.style;
        let spacing = style.window.spacing;
        let padding = wnd.layout.get_padding(&style.window);
        let panel_space = wnd.layout.calculate_usable_space(&style.window);

        let mut item_offset = 0f32;
        let mut item_width = 0f32;
        let mut item_spacing = 0f32;
        let mut panel_space = 0f32;
        
        match wnd.layout.row.ty {
            RowLayoutType::DynamicFixed => {},
            RowLayoutType::DynamicRow => {},
            RowLayoutType::DynamicFree => {},
            RowLayoutType::Dynamic => {},
            RowLayoutType::StaticFixed => {
                item_width = wnd.layout.row.item_width;
                item_offset = wnd.layout.row.index as f32 * item_width;
                item_spacing = wnd.layout.row.index as f32 * spacing.0;
            },
            RowLayoutType::StaticRow => {
                item_width = wnd.layout.row.item_width;
                item_offset = wnd.layout.row.item_offset;
                item_spacing = wnd.layout.row.index as f32 * spacing.0;
                if modify {
                    wnd.layout.row.item_offset += item_offset;
                }
            },
            RowLayoutType::StaticFree => {},
            RowLayoutType::Static => {},
        }

        let origin = float2(
            wnd.layout.cursor.0 + item_offset + item_spacing + padding.0,
            wnd.layout.cursor.1
        );
        let size = float2(item_width, wnd.layout.row.height - spacing.1);

        let bounds = Rect::new(origin, origin + size);
        if bounds.max.0 > wnd.layout.row.max_x && modify {
            wnd.layout.row.max_x = bounds.max.0;
        }
        
        bounds
    }
    
    //pub fn layout_widget(&mut self, 
}

impl Window {
    pub fn alloc_row(&mut self) {
        let spacing = float2(5f32, 5f32);
        let row_height = self.layout.row.height - spacing.1;
        
        self.layout.layout(row_height, self.layout.row.columns, spacing);
    }

    
    pub fn alloc_space(&mut self, bounds: Rect) {
        if self.layout.row.index >= self.layout.row.columns {
            self.alloc_row();
        }

        //self.layout_widget_space(bounds);
        
        self.layout.row.index += 1;
    }
}


impl Context {
    pub fn panel_begin(&mut self, title: &str, ty: PanelType) -> bool {
        let wnd_idx = if let Some(idx) = self.active {
            idx
        } else {
            return false;
        };

        let wnd = &mut self.windows[wnd_idx];
        let io = &mut self.io;

        
        if wnd.flags.contains(WindowFlags::Movable) && !wnd.flags.contains(WindowFlags::ReadOnly) {
            let style = &self.style.window.header;
            
            let mut header = wnd.bounds;
            header.max.1 = header.min.1;

            if wnd.has_header() {
                header.max.1 += 14f32 + 2f32 * style.padding.1;
                header.max.1 += 2f32 * style.label_padding.1;
            } else {
                let pane = &mut wnd.layout;
                header.max.1 += pane.get_padding(&self.style.window).1;
            }

            let mouse_down = io.is_mouse_down(MouseButton::Left);
            let mouse_inside = io.has_mouse_click_in_rect(MouseButton::Left, header);

            if mouse_down && mouse_inside {
                wnd.bounds.min += io.mouse_delta;
                wnd.bounds.max += io.mouse_delta;
                io.mouse_clicked_pos[MouseButton::Left as usize] += io.mouse_delta;
            }
        }

        // setup layout
        wnd.layout.ty = ty;
        let panel_padding = wnd.layout.get_padding(&self.style.window);
        wnd.layout.bounds = wnd.bounds.grow(
            float2(panel_padding.0, 0f32),
            float2(-panel_padding.0, 0f32)
        );
        wnd.layout.cursor = wnd.layout.bounds.min;
        wnd.layout.max_x = 0f32;
        wnd.layout.header_height = 0f32;
        wnd.layout.footer_height = 0f32;
        wnd.layout.row.index = 0;

        let style = &self.style.window.header;
        if wnd.has_header() {
            let mut header = wnd.bounds;
            header.max.1 = header.min.1;
            header.max.1 += 14f32 + 2f32 * style.padding.1;
            header.max.1 += 2f32 * style.label_padding.1;

            let h = header.height();
            wnd.layout.header_height = h;
            wnd.layout.bounds.min.1 += h;
            wnd.layout.cursor.1 += h;

            self.draw_list.add_rect_filled(header.min, header.max, 0f32, 0xff00ffff);
        }

        let mut body = wnd.bounds;
        body.min.1 += wnd.layout.header_height;
        self.draw_list.add_rect_filled(body.min, body.max, 0f32, 0xffff00ff);

        // TODO: minimized?
        true
    }

    pub fn panel_end(&mut self) {
        // TODO: scrollbars
        // TODO: panel border
        // TODO: draw resize handle
        // TODO: resize handling
    }
}
