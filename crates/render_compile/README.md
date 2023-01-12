* shader中, layout(set=100,binding=100)表示内建变量，不会被构建


# TODO
* uniform 数组构建
* 默认值与类型是否匹配的检查
* 数组类型，storage类型 to_code实现
* storage类型构建实现

* ecs + ecs调度
* 渲染系统（渲染图、shader、DrawObj、材质、process）
* Window（输入、输出）
* 声音系统
* 资产管理
* 动画系统
* Gui（布局、样式、fbo树）
* 3D
* 

buffermgr<T: group> {
	grops: vec<Bg>,
	buffer:Vec<vec<buffer>>

}

draobj
Bg{
	bind_group:,
	Vec：Buffer
	Arc(refCell<>)
}
<!-- corlor bind_group1 -> Vec<Binding -> (buffer1,offset), (buffer2,offset)>
bind_group2	(buffer3,offset), (buffer2,offset)
bind_group3	(buffer3,offset), (buffer4,offset) -->

bind_group1 + offset1 + offset2

bind_group1 + offset1 + offset2
<!-- corlor bind_group2 -> Vec<Binding -> (buffer3,offset), (buffer4,offset)> -->
