/// 带子图功能的图数据结构
/// 节点或图，只能和同属于一个父图的其他节点或图相连接


use log::debug;
use pi_hash::XHashSet;
use pi_map::vecmap::VecMap;
use pi_slotmap::{SecondaryMap, Key, SparseSecondaryMap};
use std::collections::VecDeque;

use super::graph_data::NGraph;

/// 图节点
#[derive(Debug, Clone)]
pub struct NGraphNode<K> {
    // 该节点的入度节点
    from: Vec<K>,
    // 该节点的出度节点
    to: Vec<K>,
	
}

#[derive(Debug, Clone)]
pub struct GraphNode<K: Key, T> {
    edges: NGraphNode<K>,
    parent_graph_id: K,
    index: usize,
	value: T,
	is_transfer: bool, // 是否是传输节点(不是一个真实的节点， 在生成toop图时， 需要忽略该节点， 将该节点的所有before和所有after相连)
	is_enable: bool, // 是否激活节点(如果为false其与其所有的递归前置接节点不会链接到最终的执行图上)


}

// mut query: Q<&mut>

// let r = query.get(1)
// let r = query.get(1)

#[derive(Debug)]
pub struct RootGraph<K: Key, T> {
    nodes: SecondaryMap<K, GraphNode<K, T>>,
    from: Vec<K>,
    to: Vec<K>,
	// 描述了所有的边（外部添加边时，不可以重复，使用此字段来判断是否重复）
	edges: XHashSet<(K, K)>,
	topological: Vec<K>,
	sub_graphs: SparseSecondaryMap<K, SubGraphDesc<K>>,
}

impl<K: Key, T> Default for RootGraph<K, T> {
    fn default() -> Self {
        Self { 
			nodes: Default::default(), 
			from: Default::default(), 
			to: Default::default(),
            edges: XHashSet::default(),
            topological: Vec::default(),
            sub_graphs: SparseSecondaryMap::default(),
		}
    }
}

#[derive(Debug)]
pub struct SubGraphDesc<K> {
	topological: Vec<K>,
	children_nodes: Vec<K>,
	from: Vec<K>, 
	to: Vec<K>,
}

impl<K: Key, T> RootGraph<K, T> {
    pub fn new() -> Self {
        RootGraph {
            nodes: SecondaryMap::new(),
            from: Vec::new(),
            to: Vec::new(),
			edges: XHashSet::default(),
			topological: Vec::new(),
			sub_graphs: SparseSecondaryMap::default(),
        }
    }

	/// 添加子图
	/// 当子图第一次与其他节点连接时， 子图将隶属于与其连接的节点的父图， 如果父图被删除，则此子图也会被删除
	pub fn add_sub_graph(&mut self, k: K, value: T) {
		log::trace!("graph.add_sub_graph({:?}, ())", k);
		// 子图的parent_graph_id暂时为null， 当子图第一次其他节点连接时，会重置parent_graph_id
		self.add_node(k, value, K::null());
		self.sub_graphs.insert(k, SubGraphDesc { 
			topological: Vec::new(), 
			children_nodes: Vec::new(), 
			from: Vec::new(),
			to: Vec::new(),
		});
	}

	/// 设置子图的父, 只能在该图与其他节点创建连接关系之前设置， 否则设置不成功
	pub fn set_sub_graph_parent(&mut self, k: K, parent_graph_id: K) {
		log::debug!("graph.set_sub_graph_parent({:?}, {:?})", k, parent_graph_id);
		if !parent_graph_id.is_null() {
			if let (Some(sub_node), true, true) = (self.nodes.get_mut(k), self.sub_graphs.contains_key(k), self.sub_graphs.contains_key(parent_graph_id)) {
				if sub_node.edges.from.is_empty() && sub_node.edges.to.is_empty() {
					sub_node.parent_graph_id = parent_graph_id;
					return;
				}
			}
		}

		log::debug!("set_sub_graph_parent fail, k={k:?}, parent_graph_id={parent_graph_id:?}, node.from={:?}, node.to={:?}, has_graph: {:?}, has_parent: {:?}", self.nodes.get(k).map(|n| {&n.edges.from}), self.nodes.get(k).map(|n| {&n.edges.to}), self.sub_graphs.contains_key(k), self.sub_graphs.contains_key(parent_graph_id))
	}

	/// 如果parent_graph_id是Null， 表示插入到根上
    pub fn add_node(&mut self, k: K, value: T, parent_graph_id: K) {
		log::trace!("graph.add_node({:?}, (), {:?})", k, parent_graph_id);
		debug_assert!(!self.nodes.contains_key(k));
		let mut n = GraphNode {
			edges: NGraphNode {
				from: Vec::new(),
				to: Vec::new(),
			},
			parent_graph_id,
			index: 0,
			value,
			is_transfer: false,
			is_enable: true,
		};
        if !parent_graph_id.is_null() {
			if let Some(parent_graph) = self.sub_graphs.get_mut(parent_graph_id) {
				n.index = parent_graph.children_nodes.len();
				parent_graph.children_nodes.push(k);
			}
		}
		self.nodes.insert(
            k,
            n,
        );
		
    }

	/// 设置是否为中转节点
	pub fn set_is_transfer(&mut self, k: K, is_transfer: bool) -> bool {
		if let Some(node) = self.nodes.get_mut(k) {
			if node.is_transfer  != is_transfer {
				node.is_transfer = is_transfer;
				return true;
			}
		}

		false
	}

	/// 设置是否激活节点， 默认激活
	pub fn set_enable(&mut self, k: K, is_enable: bool) -> bool {
		if let Some(node) = self.nodes.get_mut(k) {
			if node.is_enable  != is_enable {
				log::error!("graph.set_enable({:?}, {:?})", k, is_enable);
				node.is_enable = is_enable;
				return true;
			}
		}

		false
	}

	/// 取到入度节点
    pub fn before_nodes(&self, k: K) -> Option<&[K]> {
        self.nodes.get(k).map(|r| {r.edges.from.as_slice()})
    }

	/// 取到出度节点
    pub fn after_nodes(&self, k: K) -> Option<&[K]> {
        self.nodes.get(k).map(|r| {r.edges.to.as_slice()})
    }

    pub fn remove_node(&mut self, k: K) {
		log::trace!("graph.remove_node({:?})", k);
        let node = self.nodes.remove(k);
        if let Some(mut node) = node {
            for from_node in node.edges.from {
                if let Some(from) = self.nodes.get_mut(from_node) {
                    from.edges.to.retain(|&to_node| to_node != k);
					self.edges.remove(&(from_node, k));
                }
            }
            for to_node in node.edges.to {
                if let Some(to) = self.nodes.get_mut(to_node) {
                    to.edges.from.retain(|&from_node| from_node != k);
					self.edges.remove(&(k, to_node));
                }
            }
			node.edges.to = Vec::default(); // 清理图， 以防后续重复处理

            if let Some(parent_graph) = self.sub_graphs.get_mut(node.parent_graph_id) {
				log::trace!("graph.remove_node1({:?}) parent_graph_id: {:?}, {:?}", k, node.parent_graph_id, parent_graph.children_nodes.len());
                parent_graph.children_nodes.swap_remove(node.index);
				if let Some(child) = parent_graph.children_nodes.get(node.index) {
					self.nodes[*child].index = node.index;
				}
            }

			if let Some(sub_graph_desc) = self.sub_graphs.remove(k) {
				self.remove_graph(sub_graph_desc);
			}
        }
    }

    pub fn add_edge(&mut self, before: K, after: K) {
		log::trace!("graph.add_edge({:?}, {:?})", before, after);
		if let Some([before_node, after_node]) = self.nodes.get_disjoint_mut([before, after]) {
			if before_node.parent_graph_id != after_node.parent_graph_id {
				let (before_node_not_null, after_node_not_null) = (!before_node.parent_graph_id.is_null(), !after_node.parent_graph_id.is_null());
				if before_node_not_null && after_node_not_null {
					panic!("parent graph is diffrent");
				} 
				if before_node_not_null {
					// 入度、出度都为空 并且是子图，则其parent_graph_id修改为与其连接的节点的parent_graph_id
					if after_node.edges.from.is_empty() && after_node.edges.to.is_empty() && self.sub_graphs.contains_key(after) {
						after_node.parent_graph_id = before_node.parent_graph_id;
						self.sub_graphs[before_node.parent_graph_id].children_nodes.push(after);
					} else {
						panic!("parent graph is diffrent");
					}
					
				} else {
					// 入度、出度都为空 并且是子图，则其parent_graph_id修改为与其连接的节点的parent_graph_id
					if before_node.edges.from.is_empty() && before_node.edges.to.is_empty() && self.sub_graphs.contains_key(before) {
						before_node.parent_graph_id = after_node.parent_graph_id;
						self.sub_graphs[after_node.parent_graph_id].children_nodes.push(before);
					} else {
						panic!("parent graph is diffrent");
					}
					
				}
			}

			if !self.edges.insert((before, after)) {
				return;
			}
			before_node.edges.to.push(after);
			after_node.edges.from.push(before);
		};
    }

    pub fn remove_edge(&mut self, before: K, after: K) {
		log::trace!("graph.remove_edge({:?}, {:?})", before, after);
		if let Some([before_node, after_node]) = self.nodes.get_disjoint_mut([before, after]) {
			if !self.edges.remove(&(before, after)) {
				return;
			}

			before_node.edges.to.retain(|&to_node| to_node != after);
			after_node.edges.from.retain(|&from_node| from_node != before);
		};
    }

	#[inline]
	pub fn get(&mut self, k: K) -> Option<&GraphNode<K, T>> {
		self.nodes.get(k)
	}

	#[inline]
	pub fn get_graph(&mut self, k: K) -> Option<&SubGraphDesc<K>> {
		self.sub_graphs.get(k)
	}

	/// 构建图（包含子图）
	/// 1. 找到图的入度和出度
	/// 2. 对图进行topo排序
	/// 3. 如果图形成环状，返回错误
	pub fn build(&mut self) -> Result<(), Vec<K>>{
		self.from.clear();
		self.to.clear();
		self.topological.clear();

		// self.graph.topological.clear();
		let mut counts = VecMap::with_capacity(self.nodes.len());

		for sub_graph_desc in self.sub_graphs.values_mut() {
			sub_graph_desc.from.clear();
			sub_graph_desc.to.clear();
			sub_graph_desc.topological.clear();
		}
		let mut graph = self;

		// 已经处理过的节点Key
        let RootGraph{from, to, nodes, topological, sub_graphs, ..} = &mut graph;

		
        // 计算开头(入度) 和 结尾(出度) 节点
        for k in nodes.keys() {
			let v = &nodes[k];
			if !v.parent_graph_id.is_null(){
				let parent_graph = &mut sub_graphs[v.parent_graph_id];
				// 开头：没有入边的点
				if v.edges.from.is_empty() {
					parent_graph.from.push(k);
				}
	
				// 结尾：没有出边的点
				if v.edges.to.is_empty() {
					parent_graph.to.push(k);
				}
				
			} else {
				// 开头：没有入边的点
				if v.edges.from.is_empty() {
					from.push(k);
				}
	
				// 结尾：没有出边的点
				if v.edges.to.is_empty() {
					to.push(k);
				}
			}
			counts.insert(key_index(k), v.edges.from.len());
        }

        debug!("graph's from = {:?}", from);
		let mut queue: VecDeque<K> = from.iter().copied().collect::<VecDeque<K>>();
		for sub_graph in sub_graphs.values() {
			queue.extend(sub_graph.from.iter());
		}

		let mut topological_len = 0;
        while let Some(k) = queue.pop_front() {
			let node = nodes.get(k).unwrap();
			if node.parent_graph_id.is_null() {
				topological.push(k);
			} else {
				sub_graphs[node.parent_graph_id].topological.push(k);
			}
			topological_len += 1;
            
			
            // 处理 from 的 下一层
			debug!("from = {:?}, to: {:?}", k, node.edges.to);
            // 遍历节点的后续节点
            for to in &node.edges.to  {
				debug!("graph's each = {:?}, count = {:?}", to, counts[key_index(*to)]);
				counts[key_index(*to)] -= 1;
                // handle_set.insert(*to, ());
				if counts[key_index(*to)] == 0 {
					queue.push_back(*to);
				}
            }
        }

		// 如果拓扑排序列表的节点数等于图中的总节点数，则返回拓扑排序列表，否则返回空列表（说明图中存在环路）
		if topological_len == nodes.len() {
			// topological = topos;
			return Ok(());
		} else {
			topological.clear();
		}

		let keys = nodes.keys().map(|k|{k.clone()}).filter(|r| {
			let mut is_not_contains = !topological.contains(r);

			for sub_graph_desc in sub_graphs.values() {
				is_not_contains &= !sub_graph_desc.topological.contains(r);
			}

			return  is_not_contains;
		}).collect::<Vec<K>>();
		let mut iter = keys.into_iter();
		while let Some(n) = iter.next() {
			let mut cycle_keys = Vec::new();
			Self::find_cycle(nodes, n, &mut cycle_keys, Vec::new());

			if cycle_keys.len() > 0 {
				let cycle: Vec<(K, T)> = cycle_keys.iter().map(|k| {(k.clone(), nodes.remove(*k).unwrap().value)}).collect();
				pi_print_any::out_any!(log::error, "graph build error, no from node, they make cycle: {:?}", cycle);
				return Result::Err(cycle_keys);
			}
		}
		return Result::Err(Vec::new());
    }
    // 寻找循环依赖
    fn find_cycle(map: &SecondaryMap<K, GraphNode<K, T>>, node: K, nodes: &mut Vec<K>, mut indexs: Vec<usize>) {
		nodes.push(node.clone());
        indexs.push(0);
        while nodes.len() > 0 {
            let index = nodes.len() - 1;
            let k = &nodes[index];
            let n = map.get(*k).unwrap();
            let to = &n.edges.to;
            let child_index = indexs[index];
            if child_index >= to.len() {
                nodes.pop();
                indexs.pop();
                continue
            }
            let child = to[child_index].clone();
            if child == node {
                break;
            }
            indexs[index] += 1;
            nodes.push(child);
            indexs.push(0);
        }
    }


	// 递归移除子图包含的节点
	fn remove_graph(&mut self, sub_desc: SubGraphDesc<K>) {
		for child_node_id in sub_desc.children_nodes {
			let node = self.nodes.remove(child_node_id).unwrap();
			for to in node.edges.to.iter() {
				self.edges.remove(&(child_node_id, *to));
			}

			if let Some(sub_desc) = self.sub_graphs.remove(child_node_id) {
				self.remove_graph(sub_desc);
			}
		}
	}
}

impl<K: Key, T: Clone> RootGraph<K, T> {
	
	/// 生成 局部图（根据结束节点生成，没有最终到达结束节点的路径上的节点会被忽略）
	/// 生成的局部图，已经将子图中的节点摊平，摊平的意思是， 如果一个子图与某节点连接， 实际上是将子图的每个入度或每个出度与该节点相连
    pub fn gen_graph_from_keys<'a, I: Iterator<Item = &'a K>>(&self, finish: I) -> NGraph<K, T> {
		let mut part_graph = NGraph::new();
		// let mut graph_ids = XHashSet::default();
        // debug!("gen_graph_from_keys, param keys = {:?}", finish.copied().collect::<Vec<K>>());

        let mut current_keys = vec![];
		
        for k in finish{
			self.gen_node1(*k, &mut current_keys, &mut part_graph);
        }

		log::debug!("gen_graph_from_keys, current_keys = {:?}", current_keys);

		let mut from_keys = vec![];
		part_graph.to = current_keys.clone();
        while !current_keys.is_empty() {
            log::debug!("gen_graph_from_keys1, current_keys = {:?}", current_keys);

            from_keys.clear();

            for curr in current_keys.iter() {
                let curr_node = self.nodes.get(curr.clone()).unwrap();

				let from = if curr_node.edges.from.is_empty() && !curr_node.parent_graph_id.is_null() {
					if let Some(parent_graph_node) = self.nodes.get(curr_node.parent_graph_id) {
						&parent_graph_node.edges.from
					} else {
						continue;
					}
				} else {
					&curr_node.edges.from
				};

				
				self.link_from(*curr, from,  &mut from_keys, &mut part_graph);
				// 没有from, 则当前节点是图的入度节点
				let n = part_graph.nodes.get(*curr).unwrap();
				if n.from().is_empty() {
					part_graph.from.push(*curr);
				}
			}

            debug!("gen_graph_from_keys, from_keys = {:?}", from_keys);

            let _ = std::mem::swap(&mut current_keys, &mut from_keys);
        }

		// let mut graph_ids: SecondaryMap<K, bool> = SecondaryMap::default();
		// for k in self.topological.iter() {
		// 	self.build_topo(*k, &mut part_graph, &mut graph_ids);
		// }

		let mut from1: Vec<K> = part_graph.from.clone();
		let mut from: Vec<K> = Vec::new();
		let mut counts: VecMap<usize> = VecMap::with_capacity(part_graph.nodes.len());
		
		log::debug!("froms = {:?}", &from1);
		while from1.len() > 0{
			for k in from1.drain(..) {
				let node = part_graph.nodes.get(k).unwrap();
				part_graph.topological.push(k);
				
				
				// 处理 from 的 下一层
				log::debug!("from = {:?}, to: {:?}", k, node.to());
				// 遍历节点的后续节点
				for to in node.to().iter()  {
					let k = key_index(*to);
					if !counts.contains(k) {
						counts.insert(k, part_graph.nodes[*to].from().len());
					}
					
					log::debug!("graph's each = {:?}, count = {:?}", to, counts[key_index(*to)]);
					counts[key_index(*to)] -= 1;
					// handle_set.insert(*to, ());
					if counts[key_index(*to)] == 0 {
						from.push(*to);
					}
				}
			}
			part_graph.depend_split.push(part_graph.topological.len());
			std::mem::swap(&mut from1, &mut from);
		}

		part_graph
    }

	fn link_from(&self, curr: K, from: &Vec<K>, current_keys: &mut Vec<K>, part_graph: &mut NGraph<K, T>) {
		log::debug!("link_from, from = {:?}, curr = {:?}", curr, from);
		for from in from {
			if let Some(sub_graph) = self.sub_graphs.get(*from){
				log::error!("sub_graph============={:?}", (*from, curr));
				self.link_from(curr, &sub_graph.to, current_keys, part_graph);
			} else {
				let n = self.nodes.get(*from).unwrap();
				log::error!("link_from============={:?}", (*from, curr, n.is_enable));
				if (!n.is_enable) {
					return; // 未激活， 不继续处理前置节点
				}
				if n.is_transfer {
					self.link_from(curr, &n.edges.from, current_keys, part_graph);
				} else {
					self.gen_node(*from, current_keys, part_graph);
					debug!("gen_graph_from_keys, add edge = ({:?}, {:?})", from, curr);
					part_graph.add_edge(*from, curr);
					log::debug!("link_from1, from = {:?}, curr = {:?}", from, curr);
				}
			}
		}

	}

	fn gen_node1(&self, k: K, current_keys: &mut Vec<K>, part_graph: &mut NGraph<K, T>) {
		if let Some(sub_graph) = self.sub_graphs.get(k){
			for to in sub_graph.to.iter() {
				self.gen_node1(*to, current_keys, part_graph);
			}
			
		} else {
			let n = self.nodes.get(k).unwrap();
			log::error!("gen_node1============={:?}", (k, n.is_enable));
			if !n.is_enable {
				return; // 未激活， 不继续处理前置节点
			}
			if n.is_transfer { // 如果是传输节点， 继续迭代from
				for from in n.edges.from.iter() {
					self.gen_node1(*from, current_keys, part_graph);
				}
			} else if !part_graph.contains_key(k) {
				part_graph.add_node(k, n.value.clone());
				current_keys.push(k);
			}
		}
	}

	fn gen_node(&self, k: K, current_keys: &mut Vec<K>, part_graph: &mut NGraph<K, T>) {
		let n = self.nodes.get(k).unwrap();
		if !part_graph.contains_key(k) {
			part_graph.add_node(k, n.value.clone());
			current_keys.push(k);
		}
	}

	// fn build_topo(&self, k: K, part_graph: &mut NGraph<K, T>, graph_ids: &mut SecondaryMap<K, bool>) {
	// 	if let Some(g) = self.sub_graphs.get(k) {
	// 		if !graph_ids.contains_key(k) {
	// 			for i in g.topological.iter() {
	// 				self.build_topo(*i, part_graph, graph_ids);
	// 			}
	// 		} else {
	// 			graph_ids.insert(k, true);
	// 		}
	// 	} else if part_graph.contains_key(k) {
	// 		part_graph.topological.push(k);
	// 		let node = &part_graph.nodes[k];
	// 		if node.from().is_empty() {
	// 			part_graph.from.push(k)
	// 		}

	// 		if node.to().is_empty() {
	// 			part_graph.to.push(k);
	// 		}
	// 	}
	// }
}





#[inline]
pub fn key_index<K: Key>(k: K) -> usize{
	k.data().as_ffi() as u32 as usize
}

#[test]
fn test() {
	use pi_slotmap::DefaultKey;
	use pi_slotmap::SlotMap;
	use pi_null::Null;
	let mut graph = RootGraph::default();
	let mut nodes: SlotMap<DefaultKey, ()> = SlotMap::default();
	
	let nodes = [
		nodes.insert(()),
		nodes.insert(()),
		nodes.insert(()),
		nodes.insert(()),
		nodes.insert(()),
		nodes.insert(()),
		nodes.insert(()),
		nodes.insert(()),
		nodes.insert(()),
		nodes.insert(()),
		nodes.insert(()),
		nodes.insert(()),
		nodes.insert(()),
		nodes.insert(()),
		nodes.insert(()),
		nodes.insert(()),
		nodes.insert(()),
		nodes.insert(()),
		nodes.insert(()),
		nodes.insert(()),
	];

	//-----------------------根上的节点
	graph.add_node(nodes[0], (), DefaultKey::null());
	graph.add_node(nodes[1], (), DefaultKey::null());
	graph.add_node(nodes[2], (), DefaultKey::null());
	graph.add_node(nodes[3], (), DefaultKey::null());
	graph.add_node(nodes[4], (), DefaultKey::null());
	graph.add_node(nodes[5], (), DefaultKey::null());
	graph.add_node(nodes[6], (), DefaultKey::null());

	graph.add_edge(nodes[1], nodes[2]);
	graph.add_edge( nodes[2], nodes[3]);
	graph.add_edge(nodes[2], nodes[4]);

	graph.add_edge(nodes[5], nodes[6]);

	// ------------------nodes[7], nodes[8]为根上的子图图
	graph.add_sub_graph(nodes[7], ());
	graph.add_sub_graph(nodes[8], ());
	

	graph.add_edge(nodes[2], nodes[7]);
	graph.add_edge(nodes[0], nodes[8]);

	//-----------------------nodes[7]图上的子节点
	graph.add_node(nodes[9], (), nodes[7]);
	graph.add_node(nodes[10], (), nodes[7]);
	graph.add_node(nodes[11], (), nodes[7]);
	graph.add_edge(nodes[10], nodes[11]);

	//nodes[7]图上的子图nodes[12]
	graph.add_sub_graph(nodes[12], ());
	graph.add_edge(nodes[10], nodes[12]);

	//-----------------nodes[8]图上的子节点
	graph.add_node(nodes[13], (), nodes[8]);
	graph.add_node(nodes[14], (), nodes[8]);
	graph.add_node(nodes[15], (), nodes[8]);
	graph.add_edge(nodes[14], nodes[15]);


	//-----------------nodes[12]图上的子节点
	graph.add_node(nodes[16], (), nodes[12]);
	graph.add_node(nodes[17], (), nodes[12]);
	graph.add_node(nodes[18], (), nodes[12]);
	graph.add_edge(nodes[17], nodes[18]);

	// nodes[16] (nodes[12]) nodes[10] (nodes[7]) nodes[2] nodes[1]
	// nodes[15] (nodes8]) nodes[14]
	// nodes[3] nodes[2] nodes[1]
	// nodes[4] nodes[2] nodes[1]
	let finish = vec![nodes[16], nodes[15], nodes[3], nodes[4]];
	if let Err(e) =  graph.build() {
		println!("e: {:?}", e)
	};

	println!("graph: {:?}", &graph.topological);

	let g = graph.gen_graph_from_keys(finish.iter());

	println!("graph1: {:?}", g.topological, );

}


#[test]
fn test1() {
	use pi_slotmap::DefaultKey;
	use pi_slotmap::SlotMap;
	let mut graph = RootGraph::default();
	let mut nodes: SlotMap<DefaultKey, ()> = SlotMap::default();
	
	let nodes = [
		nodes.insert(()),
		nodes.insert(()),
		nodes.insert(()),
		nodes.insert(()),
		nodes.insert(()),
		nodes.insert(()),
		nodes.insert(()),
		nodes.insert(()),
		nodes.insert(()),
		nodes.insert(()),
		nodes.insert(()),
		nodes.insert(()),
		nodes.insert(()),
		nodes.insert(()),
		nodes.insert(()),
		nodes.insert(()),
		nodes.insert(()),
		nodes.insert(()),
		nodes.insert(()),
		nodes.insert(()),
	];

	graph.add_node(nodes[0], (), DefaultKey::default());     
	graph.add_node(nodes[1], (), DefaultKey::default());    
	graph.add_edge(nodes[0], nodes[1]);
	graph.add_node(nodes[2], (), DefaultKey::default());    
	graph.add_edge(nodes[0], nodes[2]) ;
	graph.add_node(nodes[3], (), DefaultKey::default());    
	graph.add_edge(nodes[0], nodes[3]);
	graph.add_node(nodes[4], (), DefaultKey::default());    
	graph.add_edge(nodes[0], nodes[4]);
	graph.add_node(nodes[5], (), DefaultKey::default());
	graph.add_edge(nodes[0], nodes[5]);
	graph.add_node(nodes[6], (), DefaultKey::default());
	graph.add_edge(nodes[0], nodes[6]);
	graph.add_node(nodes[7], (), DefaultKey::default());
	graph.add_edge(nodes[0], nodes[7]);
	graph.add_node(nodes[8], (), DefaultKey::default());
	graph.add_edge(nodes[0], nodes[8]);
	graph.add_node(nodes[9], (), DefaultKey::default());
	graph.add_edge(nodes[0], nodes[9]);
	graph.add_node(nodes[10], (), DefaultKey::default());
	graph.add_edge(nodes[0], nodes[10]);
	graph.add_edge(nodes[3], nodes[2]);
	graph.add_edge(nodes[4], nodes[2]);
	graph.add_edge(nodes[5], nodes[2]);
	graph.add_edge(nodes[6], nodes[2]);
	graph.add_edge(nodes[7], nodes[2]);
	graph.add_edge(nodes[8], nodes[2]);
	graph.add_edge(nodes[9], nodes[2]);
	graph.add_edge(nodes[10], nodes[2]);

	if let Err(e) =  graph.build() {
		println!("e: {:?}", e)
	};
	let _ = graph.build();

	println!("{:?}", &graph.topological);

}