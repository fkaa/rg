use crate::{
    Context, Window, WindowStyle, WindowFlags, MouseButton,
    
    math::{*},
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

        wnd.layout.row.ratio = 0f32;
        wnd.layout.row.filled = 0f32;
        wnd.layout.row.item_width = 0f32;
        wnd.layout.row.item_offset = 0f32;
        wnd.layout.row.columns = columns;
    }
}
