

layout(set=1,binding=0)uniform geo{
	mat4 world;
	// 扇形 SDF 信息
	// [
		//    vec3 (布局中心.x, 布局中心.y, 布局缩放.x)
		//    vec3 (布局缩放.y, sin(对称轴-y轴), cos(对称轴-y轴))
		//    vec3 (sin(边缘-对称轴), cos(边缘-对称轴), r)
	// ]
	mat4 clipSdf;// border_radius | ellipse | circle | sector | rect | border
	// 深度值， 同一个节点在不同帧之间都有很大可能不同，因此单独作为一个binding
	float depth;
	// 模糊半径（阴影使用）
	float blur;
	// 如果是渲染文字，表示文字纹理大小， 如果渲染边框，表示边框下左两个值（clipSdf中不够放）
	// 纹理尺寸
	// 由于纹理纹理的尺寸会发生改变，一旦改变，每个文字的uv会随之而变
	// 如果纹理尺寸作为uniform传入着色器，文字uv采用绝对像素的方式描述，由着色器算出最终的uv
	// 当纹理尺寸发生改变时，每个文字渲染只需要修改TextureSize即可（TextureSize）是所用文字共用的，而无须再次为每个文字创建不同的uv buffer
	vec2 textureSizeOrBottomLeftBorder;
	// 如果渲染文字，表示文字颜色，如果渲染纯色矩形，表示矩形颜色
	vec4 color;
	// 如果渲染阴影，表示阴影渲染矩形。 xy是矩形最小点的坐标，zw是矩阵最大点的坐标；注：矩形必须排除阴影半径。
	// 如果渲染文字，该字段为文字的描边颜色
	vec4 strokeColorOrURect;
};



// layout(set=1,binding=0)uniform UiMaterial{
// 	mat4 world;
// 	mat4 clipSdf;// border_radius | ellipse | circle | sector | rect | border
// 	float depth;
// 	float blur;
// 	vec2 bottomLeftBorder;// 如果是渲染边框，则需要这两个值，（clipSdf中不够放）
// 	vec4 color;
// 	vec4 uRect;// xy是矩形最小点的坐标，zw是矩阵最大点的坐标；注：矩形必须排除阴影半径。
// };


// // set 1
// layout(set = 1, binding = 0) uniform TextMaterial {
// 	mat4 world; // 世界矩阵
// 	mat4 clipSdf; // border_radius | ellipse | circle | sector | rect | border
// 	float depth; // 深度
// 	// 纹理尺寸
// 	// 由于纹理纹理的尺寸会发生改变，一旦改变，每个文字的uv会随之而变
// 	// 如果纹理尺寸作为uniform传入着色器，文字uv采用绝对像素的方式描述，由着色器算出最终的uv
// 	// 当纹理尺寸发生改变时，每个文字渲染只需要修改TextureSize即可（TextureSize）是所用文字共用的，而无须再次为每个文字创建不同的uv buffer
// 	vec2 texture_size;
// 	vec4 uColor;
// 	vec4 strokeColor;
// };