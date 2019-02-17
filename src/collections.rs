use crate::{
    Id,
};

use std::ops::{
    Index, IndexMut
};

pub union StorageValue {
    pub integer: i32,
    pub float: f32,
    pub ptr: *mut (),
}

pub struct StoragePair {
    key: Id,
    val: StorageValue,
}

impl StoragePair {
    #[inline(always)]
    pub fn new(key: Id, val: StorageValue) -> Self {
        StoragePair {
            key,
            val,
        }
    }

    #[inline(always)]
    pub fn int(key: Id, val: i32) -> Self {
        StoragePair {
            key,
            val: StorageValue {
                integer: val,
            }
        }
    }
}

pub struct IdStorage {
    pairs: Vec<StoragePair>,
}

impl IdStorage {
    pub fn new() -> Self {
        IdStorage {
            pairs: Vec::new(),
        }
    }

    fn lower_bound(&self, key: Id) -> usize {
        let len = self.pairs.len();
        if len == 0 {
            return 0;
        }
        
        let mut first = 0;
        let mut last = len - 1;

        let mut count = last - first;
        while count > 0 {
            let count2 = count >> 1;
            let offset = first + count2;
            let mid = unsafe { self.pairs.get_unchecked(offset) };
            
            if mid.key < key {
                first = offset + 1;
                count -= count2 + 1;
            } else {
                count = count2;
            }
        }

        first
    }

    pub fn get_ref(&mut self, id: Id, default_val: StorageValue) -> *mut StorageValue {
        let i = self.lower_bound(id);
        let len = self.pairs.len();

        let mut pair = unsafe { self.pairs.as_mut_ptr().offset(i as isize) };
        if i == len || unsafe { (*pair).key != id } {
            self.pairs.insert(i, StoragePair::new(id, default_val));
            pair = unsafe { self.pairs.as_mut_ptr().offset(i as isize) };
        }

        unsafe { &mut (*pair).val as *mut _ }
    }

    pub fn get_int_ref(&mut self, id: Id, default_val: i32) -> *mut i32 {
        unsafe {
            &mut (*self.get_ref(id, StorageValue {
                integer: default_val,
            })).integer as *mut _
        }
    }
    
    pub fn set_value(&mut self, id: Id, val: StorageValue) {
        let i = self.lower_bound(id);
        let len = self.pairs.len();

        let pair = unsafe { self.pairs.as_mut_ptr().offset(i as isize) };
        if i == len || unsafe { (*pair).key != id } {
            self.pairs.insert(i, StoragePair::new(id, val));
            return;
        }

        unsafe {
            (*pair).val = val;
        }
    }

    #[inline(always)]
    pub fn set_int(&mut self, id: Id, val: i32) {
        self.set_value(id, StorageValue {
            integer: val,
        });
    }
}

#[repr(transparent)]
#[derive(Copy, Clone)]
pub struct PoolIndex(usize);

pub struct Pool<T> {
    data: Vec<T>,
    map: IdStorage,
    free_index: PoolIndex,
}

impl<T: Default + Clone> Pool<T> {
    pub fn new() -> Self {
        Pool {
            data: Vec::new(),
            map: IdStorage::new(),
            free_index: PoolIndex(0),
        }
    }
    
    pub fn add(&mut self) -> PoolIndex {
        use std::mem::transmute;
        
        let idx = self.free_index;
        let len = self.data.len();
        
        if idx.0 == len {
            self.data.resize(len + 1, T::default());
            self.free_index.0 += 1;
        } else {
            let empty_slot = &self.data[idx.0];
            let free_index = unsafe { *transmute::<&T, *const u32>(empty_slot) };
            self.free_index = PoolIndex(free_index as usize);
        }

        self.data[idx.0] = T::default();
        
        idx
    }
    
    pub fn get(&mut self, id: Id) -> PoolIndex {
        let pidx = self.map.get_int_ref(id, -1);

        let pidx_val = unsafe { *pidx };

        if pidx_val != -1 {
            return PoolIndex(pidx_val as usize);
        }

        unsafe { *pidx = self.free_index.0 as i32; }
        
        self.add()
    }
}

impl<T> Index<PoolIndex> for Pool<T> {
    type Output = T;
    
    fn index(&self, index: PoolIndex) -> &T {
        &self.data[index.0]
    }
}

impl<T> IndexMut<PoolIndex> for Pool<T> {
    fn index_mut(&mut self, index: PoolIndex) -> &mut T {
        &mut self.data[index.0]
    }
}

#[test]
fn pool() {
    let mut pool = Pool::<u32>::new();
    let a = pool.get(0x1);
    let a_2 = pool.get(0x1);
    let b = pool.get(0x2);
    let c = pool.get(0x3);

    pool[a] = 100;
    pool[b] = 200;
    pool[c] = 300;

    assert_eq!(pool[a], 100);
    assert_eq!(pool[a_2], 100);
    assert_eq!(pool[b], 200);
    assert_eq!(pool[c], 300);
}

#[test]
fn storage() {
    let mut storage = IdStorage::new();

    storage.set_int(0x1, 256);
    storage.set_int(0x2, 512);
    storage.set_int(0x3, 1024);

    storage.set_int(0x3, 256);
    storage.set_int(0x2, 512);
    storage.set_int(0x1, 1024);
    
    let a = storage.get_int_ref(0x3, 404);
    let b = storage.get_int_ref(0x2, 404);
    let c = storage.get_int_ref(0x1, 404);
    let d = storage.get_int_ref(0x4, 404);

    assert_eq!(unsafe { *a }, 256);
    assert_eq!(unsafe { *b }, 512);
    assert_eq!(unsafe { *c }, 1024);
    assert_eq!(unsafe { *d }, 404);
}
