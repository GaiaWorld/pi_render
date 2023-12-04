//! 有向无环图
use std::fmt::Debug;
use pi_slotmap::{Key, SecondaryMap};

/// 图
#[derive(Debug)]
pub struct NGraph<K: Key, T> {
    // 入度为0 的 节点
    pub from: Vec<K>,

    // 出度为0 的 节点
    pub to: Vec<K>,

	// 所有节点
	pub nodes: SecondaryMap<K, NGraphNode<K, T>>,
	pub topological: Vec<K>,
}


impl<K: Key, T> NGraph<K, T>{
	pub fn new() -> Self {
		Self {
			from: Vec::default(),
			to: Default::default(),
			nodes: Default::default(),
			topological: Vec::default(),
		}
	}
	#[inline]
	pub fn from(&self) -> &[K] {
		&self.from
	}

	#[inline]
	pub fn to(&self) -> &[K] {
		&self.to
	}

	#[inline]
	pub fn get(&self, k: K) -> Option<&NGraphNode<K, T>>{
		self.nodes.get(k)
	}

	#[inline]
	pub fn contains_key(&self, k: K) -> bool {
		self.nodes.contains_key(k)
	}

	/// 如果parent_graph_id是Null， 表示插入到根上
    pub fn add_node(&mut self, k: K, value: T) {
		debug_assert!(!self.nodes.contains_key(k));
        self.nodes.insert(
            k,
            NGraphNode {
                from: Vec::new(),
                to: Vec::new(),
				_value: value,
            },
        );
    }

    pub fn add_edge(&mut self, before: K, after: K) {
		if let Some([before_node, after_node]) = self.nodes.get_disjoint_mut([before, after]) {
			before_node.to.push(after);
			after_node.from.push(before);
		};
    }
}



/// 图节点
#[derive(Debug)]
pub struct NGraphNode<K, T> {
    // 该节点的 入度节点
    from: Vec<K>,

    // 该节点的 出度节点
    to: Vec<K>,
    // // 键
    // key: K,
	_value: T
}


impl<K: Eq, T> NGraphNode<K, T>{
	pub fn from(&self) -> &[K] {
		&self.from
	}

	pub fn to(&self) -> &[K] {
		&self.to
	}
	
}