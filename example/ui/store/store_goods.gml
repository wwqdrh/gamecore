<ui>
  <!-- 商品网格页（可复用）：UIGrid 三列商品卡 + 分类过滤。
	   每页签 3x2 共 6 格（设计图），各类目数据补齐 6 件。
	   数据绑定链路：<script> 定义全量商品（含 category 字段），UIGrid
	   data="goods" 构建期填充；filter_key="category" + filter_value_var="category"
	   按本文件 category 变量过滤——被 store_tabs.gml 的
	   <Gml data-category="cat_xxx"> 具名映射覆盖后，各页签只显示自己分类的商品；
	   独立打开本文件时 category 为空串 → 不过滤显示全部（合理降级） -->
  <style>
	.goods-bg {
	  background: #f6f0de;
	  border_color: #c9b98c;
	  border_width: 2;
	  border_radius: 10;
	}
  </style>
  <script>
	// 当前分类（空串 = 不过滤，独立打开时显示全部商品）
	var category = ""

	// 全量商品数据（category 字段供过滤；price_old/countdown/sold_out 为可选状态字段）
	var goods = [
		{ category: "herb", quality: "珍品", icon: "🍄", title: "千年灵芝", price: "800" },
		{ category: "herb", quality: "上品", icon: "🏺", title: "聚气丹瓶", price: "800" },
		{ category: "herb", quality: "上品", icon: "⚔️", title: "飞剑", price: "800" },
		{ category: "herb", quality: "限时", icon: "📜", title: "天雷符箓", price: "800", price_old: "¥1200", countdown: "⏱ 04:32:10" },
		{ category: "herb", quality: "下品", icon: "🌸", title: "九转还魂草", price: "800" },
		{ category: "herb", quality: "上品", icon: "🪷", title: "玄冰雪莲", price: "800", sold_out: true },
		{ category: "tool", quality: "上品", icon: "⚔️", title: "寒光剑", price: "1200" },
		{ category: "tool", quality: "极品", icon: "🏺", title: "紫金葫芦", price: "2600" },
		{ category: "tool", quality: "下品", icon: "🪢", title: "缚灵索", price: "300" },
		{ category: "tool", quality: "珍品", icon: "🗡️", title: "诛仙剑", price: "4800" },
		{ category: "tool", quality: "上品", icon: "🪞", title: "八卦镜", price: "1500" },
		{ category: "tool", quality: "限时", icon: "🔥", title: "风火轮", price: "1800", price_old: "¥2600", countdown: "⏱ 02:15:30" },
		{ category: "skill", quality: "上品", icon: "📖", title: "吐纳诀", price: "1500" },
		{ category: "skill", quality: "下品", icon: "🌀", title: "御风术", price: "900" },
		{ category: "skill", quality: "珍品", icon: "🛡️", title: "金钟罩", price: "3200" },
		{ category: "skill", quality: "极品", icon: "📜", title: "大衍诀", price: "5200" },
		{ category: "skill", quality: "上品", icon: "💨", title: "五行遁术", price: "2000" },
		{ category: "skill", quality: "下品", icon: "🪶", title: "轻身术", price: "600" },
		{ category: "limited", quality: "限时", icon: "📜", title: "天雷符箓", price: "800", price_old: "¥1200", countdown: "⏱ 04:32:10" },
		{ category: "limited", quality: "极品", icon: "🏺", title: "紫金葫芦", price: "2600" },
		{ category: "limited", quality: "限时", icon: "🗿", title: "九阳残卷", price: "1800", price_old: "¥2600", countdown: "⏱ 01:12:45" },
		{ category: "limited", quality: "珍品", icon: "🔮", title: "混沌珠", price: "8800" },
		{ category: "limited", quality: "限时", icon: "💊", title: "丹药礼包", price: "600", price_old: "¥800", countdown: "⏱ 00:48:20" },
		{ category: "limited", quality: "上品", icon: "👝", title: "储物戒指", price: "1200", sold_out: true },
	]
  </script>
  <Panel name="GoodsPanel" class="goods-bg" anchor="full">
	<ScrollContainer name="GoodsScroll" anchor="full" margin="10">
	  <UIGrid name="GoodsGrid" columns="3" data="goods"
	         filter_key="category" filter_value_var="category"
	         h_separation="8" v_separation="8"
	         size_flags_horizontal="expand_fill" size_flags_vertical="expand_fill">
		<!-- slot 模板：商品卡条目（构建期注入，duplicate 复制） -->
		<Gml src="store_item.gml" />
	  </UIGrid>
	</ScrollContainer>
  </Panel>
</ui>
