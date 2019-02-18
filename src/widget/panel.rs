use crate::{
    Context, Window, WindowStyle, WindowFlags, MouseButton, TextAlignment,
    WidgetState, Background, Border, CursorType,
    math::{*},
};

bitflags! {
    pub struct PanelFlags: u32 {
        const None = 0;

        const Title = 1 << 0;
        const Background = 1 << 1;
        const Border = 1 << 2;
        const Styled = Self::Background.bits | Self::Border.bits;
    }
}

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
    pub ty: RowLayoutType,
    pub index: u32,
    pub height: f32,
    pub max_height: f32,

    pub max_x: f32,

    pub ratio: f32,
    pub columns: u32,
    pub item_width: f32,
    pub item_height: f32,
    pub item_offset: f32,
    pub filled: f32,
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
    pub ty: PanelType,
    pub bounds: Rect,
    pub row: RowLayout,
    pub flags: WindowFlags,
    
    pub cursor: float2,
    pub header_height: f32,
    pub footer_height: f32,
    pub max_x: f32,
    pub has_scrolling: bool,
    
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

    pub fn reset(&mut self) {
        self.row = RowLayout::new();
        self.cursor = float2(0f32, 0f32);
        self.clip = Rect::new(float2(0f32, 0f32), float2(0f32, 0f32));
    }
}

impl Context {
    pub fn panel_new_row(&mut self) {
        let idx = self.current_panel;
        let panel = &mut self.panel_stack[idx];

        let spacing = self.style.window.spacing;
        let height = panel.row.max_height;
        let row_height = height - spacing.1;

        let columns = panel.row.columns;
        
        self.panel_layout(Some(row_height), columns);
    }

    pub fn panel_cursor(&self) -> float2 {
        let idx = self.current_panel;
        let panel = &self.panel_stack[idx];
        
        panel.cursor
    }

    pub fn available_height(&mut self) -> f32 {
        let idx = self.current_panel;
        let panel = &mut self.panel_stack[idx];
        
        panel.bounds.height() - panel.cursor.1
    }

    pub fn panel_layout(&mut self, height: Option<f32>, columns: u32) {
        let idx = self.current_panel;
        let panel = &mut self.panel_stack[idx];
        
        let item_spacing = self.style.window.spacing;
        
        panel.row.index = 0;
        panel.cursor.1 += panel.row.max_height;
        panel.row.max_height = 0f32;
        panel.row.columns = columns;
        panel.row.item_offset = 0f32;

        if let Some(height) = height {
            panel.row.height = height + item_spacing.1;
        } else {
            // panel.row.height = panel.row.min_height + item_spacing.1;
        }
    }
    
    pub fn panel_alloc_space(&mut self, height: Option<f32>) -> Rect {
        let idx = self.current_panel;
        let (column, max_columns) = {
            let panel = &mut self.panel_stack[idx];

            (panel.row.index, panel.row.columns)
        };
        
        if column >= max_columns {
            self.panel_new_row();
        }

        let panel = &mut self.panel_stack[idx];
        if let Some(height) = height {
            panel.row.max_height = panel.row.max_height.max(height);
            panel.row.height = height;
        }
        let bounds = self.layout_widget_space(true);
        
        let panel = &mut self.panel_stack[idx];
        panel.row.index += 1;

        bounds
    }

    pub fn panel_peek_width(&mut self) -> f32 {
        let idx = self.current_panel;
        let panel = &mut self.panel_stack[idx];
        
        let column = panel.row.index;
        let max_columns = panel.row.columns;
        let max_height = panel.row.max_height;
        let cursor = panel.cursor;
        let item_offset = panel.row.item_offset;
        let height = panel.row.height;

        if column >= max_columns {
            self.panel_new_row();
        }
        let w = self.layout_widget_space(false).width();

        let panel = &mut self.panel_stack[idx];
        panel.row.index = column;
        panel.cursor = cursor;
        panel.row.max_height = max_height;
        panel.row.item_offset = item_offset;
        panel.row.height = height;
        
        w
    }

}

impl Context {
    pub fn panel_begin(&mut self, panel_index: usize, title: &str, ty: PanelType) -> bool {
        let panel = &mut self.panel_stack[panel_index];
        // setup layout
        panel.ty = ty;
        
        // TODO: minimized?
        true
    }

    pub fn panel_end(&mut self) {

        // TODO: scrollbars
        // TODO: panel border
        // TODO: draw resize handle
    }

    pub fn panel_index(&mut self) -> usize {
        let idx = self.panel_index;
        if idx == self.panel_stack.len() {
            self.panel_stack.push(Panel::new());
        }

        idx
    }
    
    pub fn begin_panel(&mut self, title: &str, flags: PanelFlags) -> bool {
        let (bounds, state) = self.widget(None);
        
        self.begin_panel_ex(title, Some(bounds), flags)
    }

    pub fn begin_panel_ex(&mut self, title: &str, bounds: Option<Rect>, flags: PanelFlags) -> bool {
        let bounds = if let Some(rect) = bounds {
            rect
        } else {
            let (bounds, _state) = self.widget(None);
            bounds
        };
        
        let idx = self.panel_index();
        let panel = &mut self.panel_stack[idx];
        panel.reset();

        let panel_padding = panel.get_padding(&self.style.window);
        panel.bounds = bounds.grow(
            float2(panel_padding.0, panel_padding.1),
            float2(panel_padding.0, panel_padding.1)
        );
        panel.cursor = panel.bounds.min;

        self.current_panel = self.panel_index;
        self.panel_index += 1;

        if flags.contains(PanelFlags::Background) {
            let col = crate::style::make_color(60, 59, 64, 255);
            self.draw_list.add_rect_filled(panel.bounds.min, panel.bounds.max, 0f32, col);
        }

        if flags.contains(PanelFlags::Border) {
            let border = self.style.tab.active_border;
            let border_bounds = panel.bounds.pad(-border.thickness * 0.5f32);
            self.draw_list.add_rect(border_bounds.min, border_bounds.max, border.rounding, border.thickness, border.color);
        }
        
        true
    }
    
    pub fn end_panel(&mut self) {
        self.panel_index -= 1;
        self.current_panel -= self.panel_index;
    }
}
