use crate::{
    Context, Window, WindowStyle, WindowFlags, MouseButton, TextAlignment,
    WidgetState, Background, Border, CursorType,
    math::{*},
};

mod row;

pub use self::row::*;

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
    max_height: f32,

    max_x: f32,

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
            max_height: 0f32,
            max_x: 0f32,
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
    pub fn panel_new_row(&mut self) {
        let index = self.current_index();
        let mut wnd = unsafe { self.windows.get_unchecked_mut(index) };
        
        let spacing = self.style.window.spacing;
        let height = wnd.layout.row.max_height;
        let row_height = height - spacing.1;

        let columns = wnd.layout.row.columns;
        
        self.panel_layout(index, Some(row_height), columns);
    }

    pub fn panel_cursor(&self) -> float2 {
        let index = self.current_index();
        let wnd = unsafe { self.windows.get_unchecked(index) };

        wnd.layout.cursor
    }

    pub fn panel_layout(&mut self, wnd_idx: usize, height: Option<f32>, columns: u32) {
        let mut wnd = unsafe { self.windows.get_unchecked_mut(wnd_idx) };

        let item_spacing = self.style.window.spacing;
        
        wnd.layout.row.index = 0;
        wnd.layout.cursor.1 += wnd.layout.row.max_height;
        wnd.layout.row.max_height = 0f32;
        wnd.layout.row.columns = columns;
        wnd.layout.row.item_offset = 0f32;

        if let Some(height) = height {
            wnd.layout.row.height = height + item_spacing.1;
        } else {
            // wnd.layout.row.height = wnd.layout.row.min_height + item_spacing.1;
        }
    }
    
    pub fn panel_alloc_space(&mut self, height: Option<f32>) -> Rect {
        let index = self.current_index();
        let (column, max_columns) = {
            let mut wnd = unsafe { self.windows.get_unchecked_mut(index) };

            (wnd.layout.row.index, wnd.layout.row.columns)
        };
        
        if column >= max_columns {
            self.panel_new_row();
        }

        if let Some(height) = height {
            let mut wnd = unsafe { self.windows.get_unchecked_mut(index) };
            wnd.layout.row.max_height = wnd.layout.row.max_height.max(height);
            wnd.layout.row.height = height;
        }
        let bounds = self.layout_widget_space(true);
        let mut wnd = unsafe { self.windows.get_unchecked_mut(index) };
        
        wnd.layout.row.index += 1;

        bounds
    }

    pub fn panel_peek_width(&mut self) -> f32 {
        let index = self.current_index();

        let mut wnd = unsafe { self.windows.get_unchecked(index) };
        let column = wnd.layout.row.index;
        let max_columns = wnd.layout.row.columns;
        let max_height = wnd.layout.row.max_height;
        let cursor = wnd.layout.cursor;
        let item_offset = wnd.layout.row.item_offset;
        let height = wnd.layout.row.height;

        if column >= max_columns {
            self.panel_new_row();
        }
        let w = self.layout_widget_space(false).width();

        let mut wnd = unsafe { self.windows.get_unchecked_mut(index) };
        wnd.layout.row.index = column;
        wnd.layout.cursor = cursor;
        wnd.layout.row.max_height = max_height;
        wnd.layout.row.item_offset = item_offset;
        wnd.layout.row.height = height;
        
        w
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
        }

        // setup layout
        wnd.layout.ty = ty;
        let panel_padding = wnd.layout.get_padding(&self.style.window);
        wnd.layout.bounds = wnd.bounds.grow(
            float2(panel_padding.0, panel_padding.1),
            float2(panel_padding.0, panel_padding.1)
        );
        wnd.layout.cursor = wnd.layout.bounds.min;
        wnd.layout.max_x = 0f32;
        wnd.layout.header_height = 0f32;
        wnd.layout.footer_height = 0f32;
        wnd.layout.row.index = 0;

        let state = WidgetState::None;

        let header = &self.style.window.header;
        let window = &self.style.window;
        let (border, background, header_background, text) = if state.contains(WidgetState::Active) {
            (window.active_border, window.active, header.active, header.active_text)
        } else if state.contains(WidgetState::Hovering) {
            (window.hover_border, window.hover, header.hover, header.hover_text)

        } else {
            (window.normal_border, window.normal, header.normal, header.normal_text)
        };
        
        if wnd.has_header() {
            let style = &self.style.window.header;
            let mut header = wnd.bounds;
            header.max.1 = header.min.1;
            header.max.1 += 14f32 + 2f32 * style.padding.1;
            header.max.1 += 2f32 * style.label_padding.1;

            let h = header.height();
            wnd.layout.header_height = h;
            wnd.layout.bounds.min.1 += h;
            wnd.layout.cursor.1 += h;

            match header_background {
                Background::Color(col) => {
                    self.draw_list.add_rect_filled(header.min, header.max, 0f32, col);
                },
                _ => {}
            }

            if wnd.flags.contains(WindowFlags::Title) {
                self.draw_list.add_text_wrapped(
                    &mut *self.renderer,
                    &mut self.default_font,
                    title,
                    header.min,
                    text.align,
                    header.width(),
                    text.color
                );
            }
        }

        let mut body = wnd.bounds;
        body.min.1 += wnd.layout.header_height;

        match background {
            Background::Color(col) => {
                self.draw_list.add_rect_filled(body.min, body.max, 0f32, col);
            },
            _ => {}
        }

        if border.thickness != 0f32 {
            let border_bounds = wnd.bounds.pad(-border.thickness * 0.5f32);
            self.draw_list.add_rect(border_bounds.min, border_bounds.max, border.rounding, border.thickness, border.color);
        }

        let flags = wnd.flags;
        
        /*self.row(RowType::dynamic(1));
        self.column(Some(0.8f32));
        self.paragraph(&format!("{:#?}", flags));*/
        
        // TODO: minimized?
        true
    }

    pub fn panel_end(&mut self) {
        let wnd_idx = if let Some(idx) = self.active {
            idx
        } else {
            return;
        };

        let io = &mut self.io;
        let wnd = &mut self.windows[wnd_idx];
        wnd.layout.row.max_height = 0f32;
        wnd.layout.cursor = float2(0f32, 0f32);
        wnd.layout.header_height = 0f32;
        wnd.layout.footer_height = 0f32;
        wnd.layout.max_x = 0f32;
        wnd.layout.row.height = 0f32;
        wnd.layout.row.max_height = 0f32;

        wnd.layout.row.max_x = 0f32;

        wnd.layout.row.ratio = 0f32;
        wnd.layout.row.columns = 0;
        wnd.layout.row.item_width = 0f32;
        wnd.layout.row.item_height = 0f32;
        wnd.layout.row.item_offset = 0f32;

        // drag, resize handling
        if wnd.flags.contains(WindowFlags::Movable) && !wnd.flags.contains(WindowFlags::ReadOnly) {
            let border_size = 3f32;
            let handle_size = 5f32;
            let handle_area = float2(handle_size, handle_size);

            let hori_resize_area = float2(-border_size, handle_size);
            let vertical_resize_area = float2(handle_size, -border_size);

            let handle_resize_area = float2(-handle_size, -handle_size);

            let mut header = wnd.bounds;
            header.max.1 = header.min.1;

            let style = &self.style.window.header;
            if wnd.has_header() {
                header.max.1 += 14f32 + 2f32 * style.padding.1;
                header.max.1 += 2f32 * style.label_padding.1;
            } else {
                let pane = &mut wnd.layout;
                header.max.1 += pane.get_padding(&self.style.window).1;
            }
            
            let left = Rect::new(
                wnd.bounds.min,
                float2(wnd.bounds.min.0, wnd.bounds.max.1)
            ).grow(hori_resize_area, hori_resize_area);
            let right = Rect::new(
                float2(wnd.bounds.max.0, wnd.bounds.min.1),
                wnd.bounds.max,
            ).grow(hori_resize_area, hori_resize_area);

            let top = Rect::new(
                wnd.bounds.min,
                float2(wnd.bounds.max.0, wnd.bounds.min.1)
            ).grow(vertical_resize_area, vertical_resize_area);
            let bottom = Rect::new(
                float2(wnd.bounds.min.0, wnd.bounds.max.1),
                wnd.bounds.max,
            ).grow(vertical_resize_area, vertical_resize_area);

            let tl = wnd.bounds.min;
            let tl = Rect::new(
                tl, tl
            ).grow(handle_resize_area, handle_resize_area);
            let tr = float2(wnd.bounds.max.0, wnd.bounds.min.1);
            let tr = Rect::new(
                tr, tr,
            ).grow(handle_resize_area, handle_resize_area);

            let bl = float2(wnd.bounds.min.0, wnd.bounds.max.1);
            let bl = Rect::new(
                bl, bl
            ).grow(handle_resize_area, handle_resize_area);
            let br = wnd.bounds.max;
            let br = Rect::new(
                br, br
            ).grow(handle_resize_area, handle_resize_area);
            let mouse_down = io.is_mouse_down(MouseButton::Left);

            let in_header = io.has_mouse_in_rect(header);
            
            if !io.has_mouse_in_rect(header) {
                if io.has_mouse_in_rect(left) {
                    self.cursor = CursorType::ResizeHorizontal;
                } else if io.has_mouse_in_rect(right) {
                    self.cursor = CursorType::ResizeHorizontal;
                } else if io.has_mouse_in_rect(bottom) {
                    self.cursor = CursorType::ResizeVertical;
                } else if io.has_mouse_in_rect(top) {
                    self.cursor = CursorType::ResizeVertical;
                } else if io.has_mouse_in_rect(tl) {
                    self.cursor = CursorType::ResizeNW;
                } else if io.has_mouse_in_rect(tr) {
                    self.cursor = CursorType::ResizeNE;
                } else if io.has_mouse_in_rect(bl) {
                    self.cursor = CursorType::ResizeNE;
                } else if io.has_mouse_in_rect(br) {
                    self.cursor = CursorType::ResizeNW;
                }
            }

            if mouse_down {
                if io.has_mouse_click_in_rect(MouseButton::Left, header) {
                    wnd.bounds.min += io.mouse_delta;
                    wnd.bounds.max += io.mouse_delta;
                    io.mouse_clicked_pos[MouseButton::Left as usize] += io.mouse_delta;
                } else if io.has_mouse_click_in_rect(MouseButton::Left, right) {
                    wnd.bounds.max.0 += io.mouse_delta.0;
                    io.mouse_clicked_pos[MouseButton::Left as usize].0 += io.mouse_delta.0;
                } else if io.has_mouse_click_in_rect(MouseButton::Left, left) {
                    wnd.bounds.min.0 += io.mouse_delta.0;
                    io.mouse_clicked_pos[MouseButton::Left as usize].0 += io.mouse_delta.0;
                } else if io.has_mouse_click_in_rect(MouseButton::Left, bottom) {
                    wnd.bounds.max.1 += io.mouse_delta.1;
                    io.mouse_clicked_pos[MouseButton::Left as usize].1 += io.mouse_delta.1;
                } else if io.has_mouse_click_in_rect(MouseButton::Left, top) {
                    wnd.bounds.min.1 += io.mouse_delta.1;
                    io.mouse_clicked_pos[MouseButton::Left as usize].1 += io.mouse_delta.1;
                } else if io.has_mouse_click_in_rect(MouseButton::Left, tl) {
                    wnd.bounds.min += io.mouse_delta;
                    io.mouse_clicked_pos[MouseButton::Left as usize] += io.mouse_delta;
                } else if io.has_mouse_click_in_rect(MouseButton::Left, tr) {
                    wnd.bounds.max.0 += io.mouse_delta.0;
                    wnd.bounds.min.1 += io.mouse_delta.1;
                    io.mouse_clicked_pos[MouseButton::Left as usize] += io.mouse_delta;
                } else if io.has_mouse_click_in_rect(MouseButton::Left, bl) {
                    wnd.bounds.min.0 += io.mouse_delta.0;
                    wnd.bounds.max.1 += io.mouse_delta.1;
                    io.mouse_clicked_pos[MouseButton::Left as usize] += io.mouse_delta;
                } else if io.has_mouse_click_in_rect(MouseButton::Left, br) {
                    wnd.bounds.max += io.mouse_delta;
                    io.mouse_clicked_pos[MouseButton::Left as usize] += io.mouse_delta;
                }
            }

            if io.is_mouse_down(MouseButton::Right) {
                self.draw_list.add_rect(left.min, left.max, 0f32, 1f32, 0x8800ffff);
                self.draw_list.add_rect(right.min, right.max, 0f32, 1f32, 0x8800ffff);
                self.draw_list.add_rect(top.min, top.max, 0f32, 1f32, 0x8800ffff);
                self.draw_list.add_rect(bottom.min, bottom.max, 0f32, 1f32, 0x8800ffff);


                self.draw_list.add_rect(tl.min, tl.max, 0f32, 1f32, 0x8800ff00);
                self.draw_list.add_rect(tr.min, tr.max, 0f32, 1f32, 0x8800ff00);
                self.draw_list.add_rect(bl.min, bl.max, 0f32, 1f32, 0x8800ff00);
                self.draw_list.add_rect(br.min, br.max, 0f32, 1f32, 0x8800ff00);


                self.draw_list.add_rect(header.min, header.max, 0f32, 1f32, 0x88ff00ff);
            }
        }
        // TODO: scrollbars
        // TODO: panel border
        // TODO: draw resize handle
        // TODO: resize handling
    }
}
