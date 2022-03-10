use super::window::{PiWindow, PiWindowId};
use pi_hash::XHashMap;
use std::ops::{Deref, DerefMut};

#[derive(Debug, Default)]
pub struct Windows {
    primary_id: Option<PiWindowId>,
    windows: XHashMap<PiWindowId, PiWindow>,
}

impl Windows {
    /// 第一个加入的窗口，就是 主窗口
    pub fn add(&mut self, window: PiWindow) {
        if self.windows.is_empty() {
            self.primary_id = Some(window.id());
        }
        self.windows.insert(window.id(), window);
    }

    pub fn get(&self, id: PiWindowId) -> Option<&PiWindow> {
        self.windows.get(&id).and_then(|r| Some(r.deref()))
    }

    pub fn get_mut(&mut self, id: PiWindowId) -> Option<&mut PiWindow> {
        self.windows.get_mut(&id).and_then(|r| Some(r.deref_mut()))
    }

    pub fn get_primary(&self) -> Option<&PiWindow> {
        self.get(self.primary_id.unwrap())
    }

    pub fn get_primary_mut(&mut self) -> Option<&mut PiWindow> {
        self.get_mut(self.primary_id.unwrap())
    }

    pub fn iter(&self) -> impl Iterator<Item = &PiWindow> {
        self.windows.values().map(|v| v.deref())
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut PiWindow> {
        self.windows.values_mut().map(|v| v.deref_mut())
    }
}
