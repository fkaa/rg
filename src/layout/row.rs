use crate::{
    Context, Window, WindowStyle, WindowFlags, MouseButton,
    RowLayoutType,
    
    math::{*},
};

pub enum RowType {
    DynamicRow {
        max_height: Option<f32>,
        columns: u32,
    },
    StaticRow {
        max_height: Option<f32>,
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
            max_height: None,
            columns,
        }
    }

    pub fn dynamic_ex(columns: u32, max_height: f32) -> Self {
        RowType::DynamicRow {
            max_height: Some(max_height),
            columns,
        }
    }
    
    pub fn fixed() -> Self {
        RowType::StaticRow {
            max_height: None,
        }
    }
}

impl Context {
    pub fn row(&mut self, ty: RowType) {
        let idx = self.current_panel;
        let panel = &mut self.panel_stack[idx];
        
        let (height, columns) = match ty {
            RowType::DynamicRow { max_height, columns } => {
                panel.row.ty = RowLayoutType::DynamicRow;
                panel.row.item_width = 0f32;

                (max_height, columns)
            }
            RowType::StaticRow { max_height} => {
                panel.row.ty = RowLayoutType::StaticRow;
                panel.row.item_width = 0f32;

                (max_height, u32::max_value())
            }
        };

        drop(panel);
        self.panel_layout(height, columns);
        
        let panel = &mut self.panel_stack[idx];
        panel.row.ratio = 0f32;
        panel.row.filled = 0f32;
        panel.row.item_offset = 0f32;
        panel.row.columns = columns;
    }

    pub fn layout_widget_space(&mut self, modify: bool) -> Rect {
        let idx = self.current_panel;
        let panel = &mut self.panel_stack[idx];
        
        let style = &self.style;
        let spacing = style.window.spacing;
        let padding = panel.get_padding(&style.window);
        let panel_space = panel.calculate_usable_space(&style.window);

        let mut item_offset = 0f32;
        let mut item_width = 0f32;
        let mut item_spacing = 0f32;
        
        match panel.row.ty {
            RowLayoutType::DynamicFixed => {
                let w = panel_space.max(1f32) / panel.row.columns as f32;
                item_offset = panel.row.index as f32 * w;
                item_width = w;
                item_spacing = panel.row.index as f32 * spacing.0;
            },
            RowLayoutType::DynamicRow => {
                let w = panel.row.item_width * panel_space;
                item_offset = panel.row.item_offset;
                item_width = w;

                if modify {
                    panel.row.item_offset += w + spacing.0;
                    panel.row.filled += panel.row.item_width;
                    panel.row.index = 0;
                }
            },
            RowLayoutType::DynamicFree => {},
            RowLayoutType::Dynamic => {},
            RowLayoutType::StaticFixed => {
                item_width = panel.row.item_width;
                item_offset = panel.row.index as f32 * item_width;
                item_spacing = panel.row.index as f32 * spacing.0;
            },
            RowLayoutType::StaticRow => {
                item_width = panel.row.item_width;
                item_offset = panel.row.item_offset;
                item_spacing = panel.row.index as f32 * spacing.0;
                if modify {
                    panel.row.item_offset += item_width + spacing.0;
                }
            },
            RowLayoutType::StaticFree => {},
            RowLayoutType::Static => {},
        }

        let origin = float2(
            panel.cursor.0 + item_offset + item_spacing + padding.0,
            panel.cursor.1
        );
        let size = float2(item_width, panel.row.height - spacing.1);

        let bounds = Rect::new(origin, origin + size);
        if bounds.max.0 > panel.row.max_x && modify {
            panel.row.max_x = bounds.max.0;
        }
        
        bounds
    }

    pub fn column_fixed(&mut self, width: f32) {
        let idx = self.current_panel;
        let panel = &mut self.panel_stack[idx];
        
        match panel.row.ty {
            RowLayoutType::StaticRow => {
                panel.row.item_width = width;
            }
            RowLayoutType::DynamicRow => {
                // convert to 0..1
            }
            _ => {
                panel.row.item_width = width;
            }
        }
    }

    pub fn column(&mut self, width: Option<f32>) {
        let idx = self.current_panel;
        let panel = &mut self.panel_stack[idx];
        
        match panel.row.ty {
            RowLayoutType::DynamicRow => {
                if let Some(width) = width {
                    if width + panel.row.filled > 1f32 {
                        return;
                    }

                    if width > 0f32 {
                        panel.row.item_width = width.max(0f32).min(1f32);
                    }
                } else {
                    panel.row.item_width = 1f32 - panel.row.filled;
                }
            }
            _ => {
                if let Some(width) = width {
                    panel.row.item_width = width;
                }
            }
        }
    }
}
