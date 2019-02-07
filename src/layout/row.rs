use crate::{
    Context, Window, WindowStyle, WindowFlags, MouseButton,
    
    math::{*},
};

use super::{
    RowLayoutType
};

pub enum RowType {
    DynamicRow {
        min_height: Option<f32>,
        columns: u32,
    },
    StaticRow {
        width: f32,
        min_height: Option<f32>,
        columns: u32,
    },
}


/*pub enum RowLayoutType2 {
    DynamicRow {
        min_height: Option<f32>,
        columns: u32,
    },
    StaticRow {
        width: f32,
        min_height: Option<f32>,
        columns: u32,
    },
}*/

impl RowType {
    pub fn dynamic(columns: u32) -> Self {
        RowType::DynamicRow {
            min_height: None,
            columns,
        }
    }
    
    pub fn fixed(width: f32, columns: u32) -> Self {
        RowType::StaticRow {
            width,
            min_height: None,
            columns,
        }
    }
}

impl Context {
    pub fn row(&mut self, ty: RowType) {
        let idx = if let Some(idx) = self.active {
            idx
        } else {
            return;
        };

        let mut wnd = unsafe { self.windows.get_unchecked_mut(idx) };
        let (height, columns) = match ty {
            RowType::DynamicRow { min_height, columns } => {
                wnd.layout.row.ty = RowLayoutType::DynamicRow;
                wnd.layout.row.item_width = 0f32;

                (min_height, columns)
            }
            RowType::StaticRow { width, min_height, columns } => {
                wnd.layout.row.ty = RowLayoutType::StaticRow;
                wnd.layout.row.item_width = width;

                (min_height, columns)
            }
        };

        drop(wnd);
        self.panel_layout(idx, height, columns);

        let mut wnd = unsafe { self.windows.get_unchecked_mut(idx) };
        wnd.layout.row.ratio = 0f32;
        wnd.layout.row.filled = 0f32;
        wnd.layout.row.item_offset = 0f32;
        wnd.layout.row.columns = columns;
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
        
        match wnd.layout.row.ty {
            RowLayoutType::DynamicFixed => {
                let w = panel_space.max(1f32) / wnd.layout.row.columns as f32;
                item_offset = wnd.layout.row.index as f32 * w;
                item_width = w;
                item_spacing = wnd.layout.row.index as f32 * spacing.0;
            },
            RowLayoutType::DynamicRow => {
                let w = wnd.layout.row.item_width * panel_space;
                item_offset = wnd.layout.row.item_offset;
                item_width = w;

                if modify {
                    wnd.layout.row.item_offset += w + spacing.0;
                    wnd.layout.row.filled += wnd.layout.row.item_width;
                    wnd.layout.row.index = 0;
                }
            },
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

    pub fn column(&mut self, width: Option<f32>) {
        if let Some(index) = self.active {
            let mut wnd = unsafe { self.windows.get_unchecked_mut(index) };

            match wnd.layout.row.ty {
                RowLayoutType::DynamicRow => {
                    if let Some(width) = width {
                        if width + wnd.layout.row.filled > 1f32 {
                            return;
                        }

                        if width > 0f32 {
                            wnd.layout.row.item_width = width.max(0f32).min(1f32);
                        }
                    } else {
                        wnd.layout.row.item_width = 1f32 - wnd.layout.row.filled;
                    }
                }
                _ => {
                    if let Some(width) = width {
                        wnd.layout.row.item_width = width;
                    }
                }
            }
        }
    }
}
