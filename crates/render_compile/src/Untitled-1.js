Module {
	types: {
		[1]: Type {
			name: None,
			inner: Vector {
				size: Bi,
				kind: Float,
				width: 4
			}
		},
		[2]: Type {
			name: None,
			inner: Matrix {
				columns: Quad,
				rows: Quad,
				width: 4
			}
		},
		[3]: Type {
			name: Some("CameraMatrix"),
			inner: Struct {
				members: [StructMember {
					name: Some("project"),
					ty: [2],
					binding: None,
					offset: 0
				}, StructMember {
					name: Some("view"),
					ty: [2],
					binding: None,
					offset: 64
				}],
				span: 128
			}
		},
		[4]: Type {
			name: None,
			inner: Scalar {
				kind: Float,
				width: 4
			}
		},
		[5]: Type {
			name: None,
			inner: Vector {
				size: Quad,
				kind: Float,
				width: 4
			}
		},
		[6]: Type {
			name: Some("TextMaterial"),
			inner: Struct {
				members: [StructMember {
					name: Some("world"),
					ty: [2],
					binding: None,
					offset: 0
				}, StructMember {
					name: Some("depth"),
					ty: [4],
					binding: None,
					offset: 64
				}, StructMember {
					name: Some("texture_size"),
					ty: [1],
					binding: None,
					offset: 72
				}, StructMember {
					name: Some("uColor"),
					ty: [5],
					binding: None,
					offset: 80
				}, StructMember {
					name: Some("strokeColor"),
					ty: [5],
					binding: None,
					offset: 96
				}],
				span: 112
			}
		},
		[7]: Type {
			name: None,
			inner: Vector {
				size: Tri,
				kind: Float,
				width: 4
			}
		},
		[8]: Type {
			name: None,
			inner: Struct {
				members: [StructMember {
					name: Some("vUv"),
					ty: [1],
					binding: Some(Location {
						location: 0,
						interpolation: Some(Perspective),
						sampling: None
					}),
					offset: 0
				}, StructMember {
					name: None,
					ty: [5],
					binding: Some(BuiltIn(Position)),
					offset: 8
				}],
				span: 24
			}
		}
	},
	constants: {
		[1]: Constant {
			name: None,
			specialization: None,
			inner: Scalar {
				width: 4,
				value: Sint(0)
			}
		},
		[2]: Constant {
			name: None,
			specialization: None,
			inner: Scalar {
				width: 4,
				value: Sint(1)
			}
		},
		[3]: Constant {
			name: None,
			specialization: None,
			inner: Scalar {
				width: 4,
				value: Float(1.0)
			}
		},
		[4]: Constant {
			name: None,
			specialization: None,
			inner: Scalar {
				width: 4,
				value: Float(0.5)
			}
		},
		[5]: Constant {
			name: None,
			specialization: None,
			inner: Scalar {
				width: 4,
				value: Float(60000.0)
			}
		}
	},
	global_variables: {
		[1]: GlobalVariable {
			name: Some("position"),
			class: Private,
			binding: None,
			ty: [1],
			init: None
		},
		[2]: GlobalVariable {
			name: Some("uv"),
			class: Private,
			binding: None,
			ty: [1],
			init: None
		},
		[3]: GlobalVariable {
			name: Some("vUv"),
			class: Private,
			binding: None,
			ty: [1],
			init: None
		},
		[4]: GlobalVariable {
			name: None,
			class: Uniform,
			binding: Some(ResourceBinding {
				group: 0,
				binding: 0
			}),
			ty: [3],
			init: None
		},
		[5]: GlobalVariable {
			name: None,
			class: Uniform,
			binding: Some(ResourceBinding {
				group: 1,
				binding: 0
			}),
			ty: [6],
			init: None
		},
		[6]: GlobalVariable {
			name: Some("gl_Position"),
			class: Private,
			binding: None,
			ty: [5],
			init: None
		}
	},
	functions: {
		[1]: Function {
			name: Some("main"),
			arguments: [],
			result: None,
			local_variables: {
				[1]: LocalVariable {
					name: Some("p"),
					ty: [5],
					init: None
				}
			},
			expressions: {
				[1]: GlobalVariable([1]),
				[2]: GlobalVariable([2]),
				[3]: GlobalVariable([3]),
				[4]: GlobalVariable([4]),
				[5]: AccessIndex {
					base: [4],
					index: 0
				},
				[6]: GlobalVariable([4]),
				[7]: AccessIndex {
					base: [6],
					index: 1
				},
				[8]: GlobalVariable([5]),
				[9]: AccessIndex {
					base: [8],
					index: 0
				},
				[10]: GlobalVariable([5]),
				[11]: AccessIndex {
					base: [10],
					index: 1
				},
				[12]: GlobalVariable([5]),
				[13]: AccessIndex {
					base: [12],
					index: 2
				},
				[14]: GlobalVariable([5]),
				[15]: AccessIndex {
					base: [14],
					index: 3
				},
				[16]: GlobalVariable([5]),
				[17]: AccessIndex {
					base: [16],
					index: 4
				},
				[18]: Load {
					pointer: [7]
				},
				[19]: Load {
					pointer: [9]
				},
				[20]: Binary {
					op: Multiply,
					left: [18],
					right: [19]
				},
				[21]: Load {
					pointer: [1]
				},
				[22]: AccessIndex {
					base: [21],
					index: 0
				},
				[23]: Load {
					pointer: [1]
				},
				[24]: AccessIndex {
					base: [23],
					index: 1
				},
				[25]: Constant([3]),
				[26]: Constant([3]),
				[27]: Compose {
					ty: [5],
					components: [
						[22],
						[24],
						[25],
						[26]
					]
				},
				[28]: Binary {
					op: Multiply,
					left: [20],
					right: [27]
				},
				[29]: LocalVariable([1]),
				[30]: GlobalVariable([6]),
				[31]: Load {
					pointer: [5]
				},
				[32]: Load {
					pointer: [29]
				},
				[33]: AccessIndex {
					base: [32],
					index: 0
				},
				[34]: Constant([4]),
				[35]: Binary {
					op: Add,
					left: [33],
					right: [34]
				},
				[36]: Load {
					pointer: [29]
				},
				[37]: AccessIndex {
					base: [36],
					index: 0
				},
				[38]: Constant([4]),
				[39]: Binary {
					op: Add,
					left: [37],
					right: [38]
				},
				[40]: Math {
					fun: Floor,
					arg: [39],
					arg1: None,
					arg2: None,
					arg3: None
				},
				[41]: Load {
					pointer: [29]
				},
				[42]: AccessIndex {
					base: [41],
					index: 1
				},
				[43]: Constant([4]),
				[44]: Binary {
					op: Add,
					left: [42],
					right: [43]
				},
				[45]: Load {
					pointer: [29]
				},
				[46]: AccessIndex {
					base: [45],
					index: 1
				},
				[47]: Constant([4]),
				[48]: Binary {
					op: Add,
					left: [46],
					right: [47]
				},
				[49]: Math {
					fun: Floor,
					arg: [48],
					arg1: None,
					arg2: None,
					arg3: None
				},
				[50]: Constant([3]),
				[51]: Constant([3]),
				[52]: Compose {
					ty: [5],
					components: [
						[40],
						[49],
						[50],
						[51]
					]
				},
				[53]: Binary {
					op: Multiply,
					left: [31],
					right: [52]
				},
				[54]: AccessIndex {
					base: [30],
					index: 2
				},
				[55]: Load {
					pointer: [11]
				},
				[56]: Constant([5]),
				[57]: Binary {
					op: Divide,
					left: [55],
					right: [56]
				},
				[58]: Load {
					pointer: [2]
				},
				[59]: Load {
					pointer: [13]
				},
				[60]: Binary {
					op: Divide,
					left: [58],
					right: [59]
				}
			},
			named_expressions: {},
			body: Block {
				body: [Emit([5. .5]), Emit([7. .7]), Emit([9. .9]), Emit([11. .11]), Emit([13. .13]), Emit([15. .15]), Emit([17. .24]), Emit([27. .28]), Store {
					pointer: [29],
					value: [28]
				}, Emit([31. .33]), Emit([35. .37]), Emit([39. .42]), Emit([44. .46]), Emit([48. .49]), Emit([52. .53]), Store {
					pointer: [30],
					value: [53]
				}, Emit([54. .55]), Emit([57. .57]), Store {
					pointer: [54],
					value: [57]
				}, Emit([58. .60]), Store {
					pointer: [3],
					value: [60]
				}, Return {
					value: None
				}]
			}
		}
	},
	entry_points: [EntryPoint {
		name: "main",
		stage: Vertex,
		early_depth_test: None,
		workgroup_size: [0, 0, 0],
		function: Function {
			name: None,
			arguments: [FunctionArgument {
				name: Some("position"),
				ty: [1],
				binding: Some(Location {
					location: 0,
					interpolation: Some(Perspective),
					sampling: None
				})
			}, FunctionArgument {
				name: Some("uv"),
				ty: [1],
				binding: Some(Location {
					location: 1,
					interpolation: Some(Perspective),
					sampling: None
				})
			}],
			result: Some(FunctionResult {
				ty: [8],
				binding: None
			}),
			local_variables: {},
			expressions: {
				[1]: GlobalVariable([1]),
				[2]: GlobalVariable([1]),
				[3]: GlobalVariable([2]),
				[4]: GlobalVariable([2]),
				[5]: GlobalVariable([3]),
				[6]: GlobalVariable([3]),
				[7]: GlobalVariable([4]),
				[8]: AccessIndex {
					base: [7],
					index: 0
				},
				[9]: GlobalVariable([4]),
				[10]: AccessIndex {
					base: [9],
					index: 1
				},
				[11]: GlobalVariable([5]),
				[12]: AccessIndex {
					base: [11],
					index: 0
				},
				[13]: GlobalVariable([5]),
				[14]: AccessIndex {
					base: [13],
					index: 1
				},
				[15]: GlobalVariable([5]),
				[16]: AccessIndex {
					base: [15],
					index: 2
				},
				[17]: GlobalVariable([5]),
				[18]: AccessIndex {
					base: [17],
					index: 3
				},
				[19]: GlobalVariable([5]),
				[20]: AccessIndex {
					base: [19],
					index: 4
				},
				[21]: GlobalVariable([1]),
				[22]: FunctionArgument(0),
				[23]: GlobalVariable([2]),
				[24]: FunctionArgument(1),
				[25]: GlobalVariable([3]),
				[26]: Load {
					pointer: [25]
				},
				[27]: GlobalVariable([6]),
				[28]: Load {
					pointer: [27]
				},
				[29]: Compose {
					ty: [8],
					components: [
						[26],
						[28]
					]
				}
			},
			named_expressions: {},
			body: Block {
				body: [Store {
					pointer: [21],
					value: [22]
				}, Store {
					pointer: [23],
					value: [24]
				}, Emit([8. .8]), Emit([10. .10]), Emit([12. .12]), Emit([14. .14]), Emit([16. .16]), Emit([18. .18]), Call {
					function: [1],
					arguments: [],
					result: None
				}, Emit([26. .26]), Emit([28. .28]), Emit([29. .29]), Return {
					value: Some([29])
				}]
			}
		}
	}]
}
Module {
	types: {
		[1]: Type {
			name: None,
			inner: Vector {
				size: Bi,
				kind: Float,
				width: 4
			}
		},
		[2]: Type {
			name: None,
			inner: Vector {
				size: Quad,
				kind: Float,
				width: 4
			}
		},
		[3]: Type {
			name: None,
			inner: Matrix {
				columns: Quad,
				rows: Quad,
				width: 4
			}
		},
		[4]: Type {
			name: None,
			inner: Scalar {
				kind: Float,
				width: 4
			}
		},
		[5]: Type {
			name: Some("TextMaterial"),
			inner: Struct {
				members: [StructMember {
					name: Some("world"),
					ty: [3],
					binding: None,
					offset: 0
				}, StructMember {
					name: Some("depth"),
					ty: [4],
					binding: None,
					offset: 64
				}, StructMember {
					name: Some("texture_size"),
					ty: [1],
					binding: None,
					offset: 72
				}, StructMember {
					name: Some("uColor"),
					ty: [2],
					binding: None,
					offset: 80
				}, StructMember {
					name: Some("strokeColor"),
					ty: [2],
					binding: None,
					offset: 96
				}],
				span: 112
			}
		},
		[6]: Type {
			name: None,
			inner: Sampler {
				comparison: false
			}
		},
		[7]: Type {
			name: None,
			inner: Image {
				dim: D2,
				arrayed: false,
				class: Sampled {
					kind: Float,
					multi: false
				}
			}
		},
		[8]: Type {
			name: None,
			inner: Image {
				dim: D1,
				arrayed: false,
				class: Sampled {
					kind: Float,
					multi: false
				}
			}
		},
		[9]: Type {
			name: None,
			inner: Image {
				dim: Cube,
				arrayed: false,
				class: Sampled {
					kind: Float,
					multi: false
				}
			}
		},
		[10]: Type {
			name: None,
			inner: Vector {
				size: Tri,
				kind: Float,
				width: 4
			}
		},
		[11]: Type {
			name: None,
			inner: Image {
				dim: D3,
				arrayed: false,
				class: Sampled {
					kind: Float,
					multi: false
				}
			}
		},
		[12]: Type {
			name: None,
			inner: Image {
				dim: D1,
				arrayed: false,
				class: Depth {
					multi: false
				}
			}
		},
		[13]: Type {
			name: None,
			inner: Image {
				dim: D2,
				arrayed: false,
				class: Depth {
					multi: false
				}
			}
		},
		[14]: Type {
			name: None,
			inner: Image {
				dim: Cube,
				arrayed: false,
				class: Depth {
					multi: false
				}
			}
		},
		[15]: Type {
			name: None,
			inner: Image {
				dim: D1,
				arrayed: true,
				class: Sampled {
					kind: Float,
					multi: false
				}
			}
		},
		[16]: Type {
			name: None,
			inner: Image {
				dim: D2,
				arrayed: true,
				class: Sampled {
					kind: Float,
					multi: false
				}
			}
		},
		[17]: Type {
			name: None,
			inner: Image {
				dim: Cube,
				arrayed: true,
				class: Sampled {
					kind: Float,
					multi: false
				}
			}
		},
		[18]: Type {
			name: None,
			inner: Image {
				dim: D1,
				arrayed: true,
				class: Depth {
					multi: false
				}
			}
		},
		[19]: Type {
			name: None,
			inner: Image {
				dim: D2,
				arrayed: true,
				class: Depth {
					multi: false
				}
			}
		},
		[20]: Type {
			name: None,
			inner: Image {
				dim: Cube,
				arrayed: true,
				class: Depth {
					multi: false
				}
			}
		},
		[21]: Type {
			name: None,
			inner: Struct {
				members: [StructMember {
					name: Some("o_Target"),
					ty: [2],
					binding: Some(Location {
						location: 0,
						interpolation: Some(Perspective),
						sampling: None
					}),
					offset: 0
				}],
				span: 16
			}
		}
	},
	constants: {
		[1]: Constant {
			name: None,
			specialization: None,
			inner: Scalar {
				width: 4,
				value: Sint(0)
			}
		},
		[2]: Constant {
			name: None,
			specialization: None,
			inner: Scalar {
				width: 4,
				value: Sint(1)
			}
		},
		[3]: Constant {
			name: None,
			specialization: None,
			inner: Scalar {
				width: 4,
				value: Sint(2)
			}
		},
		[4]: Constant {
			name: None,
			specialization: None,
			inner: Scalar {
				width: 4,
				value: Float(1.0)
			}
		}
	},
	global_variables: {
		[1]: GlobalVariable {
			name: Some("vUv"),
			class: Private,
			binding: None,
			ty: [1],
			init: None
		},
		[2]: GlobalVariable {
			name: Some("o_Target"),
			class: Private,
			binding: None,
			ty: [2],
			init: None
		},
		[3]: GlobalVariable {
			name: None,
			class: Uniform,
			binding: Some(ResourceBinding {
				group: 1,
				binding: 0
			}),
			ty: [5],
			init: None
		},
		[4]: GlobalVariable {
			name: Some("samp"),
			class: Handle,
			binding: Some(ResourceBinding {
				group: 2,
				binding: 0
			}),
			ty: [6],
			init: None
		},
		[5]: GlobalVariable {
			name: Some("tex2d"),
			class: Handle,
			binding: Some(ResourceBinding {
				group: 2,
				binding: 1
			}),
			ty: [7],
			init: None
		}
	},
	functions: {
		[1]: Function {
			name: Some("main"),
			arguments: [],
			result: None,
			local_variables: {
				[1]: LocalVariable {
					name: Some("c"),
					ty: [2],
					init: None
				},
				[2]: LocalVariable {
					name: Some("samp"),
					ty: [2],
					init: None
				}
			},
			expressions: {
				[1]: GlobalVariable([1]),
				[2]: GlobalVariable([2]),
				[3]: GlobalVariable([3]),
				[4]: AccessIndex {
					base: [3],
					index: 0
				},
				[5]: GlobalVariable([3]),
				[6]: AccessIndex {
					base: [5],
					index: 1
				},
				[7]: GlobalVariable([3]),
				[8]: AccessIndex {
					base: [7],
					index: 2
				},
				[9]: GlobalVariable([3]),
				[10]: AccessIndex {
					base: [9],
					index: 3
				},
				[11]: GlobalVariable([3]),
				[12]: AccessIndex {
					base: [11],
					index: 4
				},
				[13]: GlobalVariable([4]),
				[14]: GlobalVariable([5]),
				[15]: Load {
					pointer: [10]
				},
				[16]: LocalVariable([1]),
				[17]: Load {
					pointer: [1]
				},
				[18]: Load {
					pointer: [1]
				},
				[19]: ImageSample {
					image: [14],
					sampler: [13],
					gather: None,
					coordinate: [18],
					array_index: None,
					offset: None,
					level: Auto,
					depth_ref: None
				},
				[20]: LocalVariable([2]),
				[21]: AccessIndex {
					base: [16],
					index: 3
				},
				[22]: Load {
					pointer: [16]
				},
				[23]: AccessIndex {
					base: [22],
					index: 3
				},
				[24]: Constant([4]),
				[25]: Load {
					pointer: [20]
				},
				[26]: AccessIndex {
					base: [25],
					index: 2
				},
				[27]: Binary {
					op: Subtract,
					left: [24],
					right: [26]
				},
				[28]: Binary {
					op: Multiply,
					left: [23],
					right: [27]
				},
				[29]: Load {
					pointer: [16]
				}
			},
			named_expressions: {},
			body: Block {
				body: [Emit([4. .4]), Emit([6. .6]), Emit([8. .8]), Emit([10. .10]), Emit([12. .12]), Emit([15. .15]), Store {
					pointer: [16],
					value: [15]
				}, Emit([17. .19]), Store {
					pointer: [20],
					value: [19]
				}, Emit([21. .23]), Emit([25. .28]), Store {
					pointer: [21],
					value: [28]
				}, Emit([29. .29]), Store {
					pointer: [2],
					value: [29]
				}, Return {
					value: None
				}]
			}
		}
	},
	entry_points: [EntryPoint {
		name: "main",
		stage: Fragment,
		early_depth_test: None,
		workgroup_size: [0, 0, 0],
		function: Function {
			name: None,
			arguments: [FunctionArgument {
				name: Some("vUv"),
				ty: [1],
				binding: Some(Location {
					location: 0,
					interpolation: Some(Perspective),
					sampling: None
				})
			}],
			result: Some(FunctionResult {
				ty: [21],
				binding: None
			}),
			local_variables: {},
			expressions: {
				[1]: GlobalVariable([1]),
				[2]: GlobalVariable([1]),
				[3]: GlobalVariable([2]),
				[4]: GlobalVariable([2]),
				[5]: GlobalVariable([3]),
				[6]: AccessIndex {
					base: [5],
					index: 0
				},
				[7]: GlobalVariable([3]),
				[8]: AccessIndex {
					base: [7],
					index: 1
				},
				[9]: GlobalVariable([3]),
				[10]: AccessIndex {
					base: [9],
					index: 2
				},
				[11]: GlobalVariable([3]),
				[12]: AccessIndex {
					base: [11],
					index: 3
				},
				[13]: GlobalVariable([3]),
				[14]: AccessIndex {
					base: [13],
					index: 4
				},
				[15]: GlobalVariable([4]),
				[16]: GlobalVariable([4]),
				[17]: GlobalVariable([5]),
				[18]: GlobalVariable([5]),
				[19]: GlobalVariable([1]),
				[20]: FunctionArgument(0),
				[21]: GlobalVariable([2]),
				[22]: Load {
					pointer: [21]
				},
				[23]: Compose {
					ty: [21],
					components: [
						[22]
					]
				}
			},
			named_expressions: {},
			body: Block {
				body: [Store {
					pointer: [19],
					value: [20]
				}, Emit([6. .6]), Emit([8. .8]), Emit([10. .10]), Emit([12. .12]), Emit([14. .14]), Call {
					function: [1],
					arguments: [],
					result: None
				}, Emit([22. .22]), Emit([23. .23]), Return {
					value: Some([23])
				}]
			}
		}
	}]
}