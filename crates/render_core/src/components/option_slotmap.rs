use pi_slotmap::{Key, SlotMap};

pub struct OptionSlotMap<K: Key, V>(SlotMap<K, Option<V>>);

impl<K: Key, V> Default for OptionSlotMap<K, V> {
    fn default() -> Self {
        let map = SlotMap::<K, Option<V>>::default();
        Self(map)
    }
}

impl<K: Key, V> OptionSlotMap<K, V> {
    /// 添加
    pub fn insert(&mut self, v: Option<V>) -> K {
        self.0.insert(v)
    }

    /// 移除
    pub fn remove(&mut self, key: K) -> Option<V> {
        let r = self.0.remove(key);
        r.and_then(|v| v)
    }

    /// 取引用
    pub fn get(&self, key: K) -> Option<&V> {
        let r = self.0.get(key);
        r.and_then(|v| v.as_ref())
    }

    /// 取可变引用
    pub fn get_mut(&mut self, key: K) -> Option<&mut V> {
        let r = self.0.get_mut(key);
        r.and_then(|v| v.as_mut())
    }

    /// 迭代器
    pub fn iter(&self) -> impl Iterator<Item = (K, &V)> {
        let r = self.0.iter();
        let r = r.filter(|(_, v)| (*v).is_none());
        r.map(|(k, v)| (k, (*v).as_ref().unwrap()))
    }

    /// 可变迭代器
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (K, &mut V)> {
        let r = self.0.iter_mut();
        let r = r.filter(|(_, v)| (*v).is_none());
        r.map(|(k, v)| (k, (*v).as_mut().unwrap()))
    }
}
