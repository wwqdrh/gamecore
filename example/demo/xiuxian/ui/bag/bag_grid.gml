<ui script="bag_grid.gd">
  <!-- 背包物品网格（可复用）：UIGrid 8 列 x 6 行 + 分类过滤 + 点选联动详情。
	   数据绑定链路：<script> 定义全量物品（category 供分类过滤，name/desc
	   为完整物品字段——数据自带展示信息，消费方不做映射补全），UIGrid
	   data="goods" 构建期填充；filter_key="category" + filter_value_var="category"
	   按本文件 category 变量过滤——初始值 "pill"（与 bag_panel.gd 初始状态一致）；
	   运行时过滤由状态下行驱动：网格监听 panel.category_changed 后
	   set_meta("__filter_value") + update 全量数据重刷；空串 = 不过滤（"全部"）。
	   点选链路（命令上行）：UIGrid @s_click_item 回调本文件脚本，
	   条目选中互斥（selected 契约）+ panel.select_item(完整条目数据) 上报 -->
  <style>
	.grid-bg {
	  background: #e9e2c8;
	  border_color: #c9b98c;
	  border_width: 2;
	  border_radius: 10;
	}
  </style>
  <script>
	// 当前分类（"pill" = 丹药页初始选中；运行时以 panel.category 状态为准）
	var category = "pill"

	// 全量物品数据（48 格 = 8 列 x 6 行）：category 供分类过滤，
	// name/desc 为完整物品字段（真实项目由物品配置表提供），
	// quality "稀有" 紫底；selected 选中金边；locked 锁定格
	var goods = [
		{ category: "pill", icon: "🏺", count: "x1", name: "聚气丹", desc: "服用后可快速聚集天地灵气，提升修为。是修仙者常用的助修丹药。", selected: true },
		{ category: "pill", icon: "⚔️", count: "x1", name: "精铁剑", desc: "修士常用的法器长剑，灌注灵力可斩妖除魔。", quality: "稀有" },
		{ category: "pill", icon: "🌿", count: "x16", name: "清心草", desc: "三阶灵草，炼制清心丹的主药，可安神静气。" },
		{ category: "pill", icon: "🍶", count: "x5", name: "灵酒", desc: "以灵果酿制的灵酒，饮之灵气充盈，微醺入定。" },
		{ category: "pill", icon: "📜", count: "x6", name: "符篆", desc: "手绘符篆，激发后可释放一次法术，用后即毁。" },
		{ category: "pill", icon: "💎", count: "x24", name: "灵石", desc: "修仙界通用货币，蕴含精纯灵气，可辅助修炼。" },
		{ category: "pill", icon: "🌿", count: "x16", name: "清心草", desc: "三阶灵草，炼制清心丹的主药，可安神静气。" },
		{ category: "pill", icon: "🪷", count: "x4", name: "碧水莲", desc: "生于灵潭深处的三阶灵莲，百年一开花，珍稀难得。", quality: "稀有" },
		{ category: "pill", icon: "🍶", count: "x35", name: "灵酒", desc: "以灵果酿制的灵酒，饮之灵气充盈，微醺入定。" },
		{ category: "pill", icon: "⚔️", count: "x1", name: "精铁剑", desc: "修士常用的法器长剑，灌注灵力可斩妖除魔。" },
		{ category: "pill", icon: "🌸", count: "x26", name: "灵桃花", desc: "桃灵树所结之花，可入药，亦可酿制桃花酿。" },
		{ category: "pill", icon: "🍶", count: "x15", name: "灵酒", desc: "以灵果酿制的灵酒，饮之灵气充盈，微醺入定。", quality: "稀有" },
		{ category: "pill", icon: "🧪", count: "x6", name: "凝魂露", desc: "炼丹辅药，可提升丹药成丹率，炼丹师必备。" },
		{ category: "pill", icon: "📜", count: "x3", name: "符篆", desc: "手绘符篆，激发后可释放一次法术，用后即毁。" },
		{ category: "pill", icon: "🌿", count: "x33", name: "清心草", desc: "三阶灵草，炼制清心丹的主药，可安神静气。" },
		{ category: "pill", icon: "💎", count: "x11", name: "灵石", desc: "修仙界通用货币，蕴含精纯灵气，可辅助修炼。" },
		{ category: "pill", icon: "🍄", count: "x11", name: "千年灵芝", desc: "千年灵芝，炼制培元丹不可或缺的药材。", quality: "稀有" },
		{ category: "pill", icon: "⚔️", count: "x3", name: "精铁剑", desc: "修士常用的法器长剑，灌注灵力可斩妖除魔。" },
		{ category: "pill", icon: "🪙", count: "x3", name: "古灵钱", desc: "前朝遗留的灵钱，据说串成串可以布聚灵阵。", quality: "稀有" },
		{ category: "pill", icon: "🍶", count: "x23", name: "灵酒", desc: "以灵果酿制的灵酒，饮之灵气充盈，微醺入定。" },
		{ category: "pill", icon: "🌸", count: "x16", name: "灵桃花", desc: "桃灵树所结之花，可入药，亦可酿制桃花酿。" },
		{ category: "pill", icon: "🍶", count: "x3", name: "灵酒", desc: "以灵果酿制的灵酒，饮之灵气充盈，微醺入定。" },
		{ category: "pill", icon: "🌸", count: "x31", name: "灵桃花", desc: "桃灵树所结之花，可入药，亦可酿制桃花酿。" },
		{ category: "pill", icon: "🍶", count: "x9", name: "灵酒", desc: "以灵果酿制的灵酒，饮之灵气充盈，微醺入定。" },
		{ category: "pill", icon: "📜", count: "x16", name: "符篆", desc: "手绘符篆，激发后可释放一次法术，用后即毁。" },
		{ category: "pill", icon: "🌿", count: "x24", name: "清心草", desc: "三阶灵草，炼制清心丹的主药，可安神静气。" },
		{ category: "pill", icon: "⚔️", count: "x3", name: "精铁剑", desc: "修士常用的法器长剑，灌注灵力可斩妖除魔。" },
		{ category: "pill", icon: "🍶", count: "x10", name: "灵酒", desc: "以灵果酿制的灵酒，饮之灵气充盈，微醺入定。" },
		{ category: "pill", icon: "🌸", count: "x26", name: "灵桃花", desc: "桃灵树所结之花，可入药，亦可酿制桃花酿。" },
		{ category: "pill", icon: "📜", count: "x6", name: "符篆", desc: "手绘符篆，激发后可释放一次法术，用后即毁。" },
		{ category: "pill", icon: "💧", count: "x14", name: "净灵水", desc: "经灵泉过滤的纯净水，洗炼法器必备。" },
		{ category: "pill", icon: "🌿", count: "x17", name: "清心草", desc: "三阶灵草，炼制清心丹的主药，可安神静气。" },
		{ category: "pill", icon: "🍶", count: "x7", name: "灵酒", desc: "以灵果酿制的灵酒，饮之灵气充盈，微醺入定。", quality: "稀有" },
		{ category: "pill", icon: "🌺", count: "x16", name: "赤焰花", desc: "性烈属火，是火系丹药的常用引子。" },
		{ category: "pill", icon: "⚔️", count: "x1", name: "精铁剑", desc: "修士常用的法器长剑，灌注灵力可斩妖除魔。" },
		{ category: "pill", icon: "🍶", count: "x4", name: "灵酒", desc: "以灵果酿制的灵酒，饮之灵气充盈，微醺入定。" },
		{ category: "pill", icon: "📜", count: "x12", name: "符篆", desc: "手绘符篆，激发后可释放一次法术，用后即毁。" },
		{ category: "pill", icon: "🥬", count: "x26", name: "翠灵菜", desc: "灵田所产灵蔬，服食可小补灵气。" },
		{ category: "pill", icon: "⚔️", count: "x2", name: "精铁剑", desc: "修士常用的法器长剑，灌注灵力可斩妖除魔。", quality: "稀有" },
		{ category: "pill", icon: "💎", count: "x3", name: "灵石", desc: "修仙界通用货币，蕴含精纯灵气，可辅助修炼。" },
		{ category: "pill", icon: "🌼", count: "x21", name: "金盏菊", desc: "二阶灵花，晒干后可泡制灵茶，清目养神。" },
		{ category: "pill", icon: "🥕", count: "x16", name: "土灵参", desc: "形似萝卜的灵参，药性温和，适合筑基期修士。" },
		{ category: "pill", icon: "🍶", count: "x6", name: "灵酒", desc: "以灵果酿制的灵酒，饮之灵气充盈，微醺入定。" },
		{ category: "pill", icon: "🌿", count: "x33", name: "清心草", desc: "三阶灵草，炼制清心丹的主药，可安神静气。" },
		{ category: "pill", icon: "🍄", count: "x4", name: "千年灵芝", desc: "千年灵芝，炼制培元丹不可或缺的药材。", quality: "稀有" },
		{ category: "pill", icon: "🍶", count: "x11", name: "灵酒", desc: "以灵果酿制的灵酒，饮之灵气充盈，微醺入定。" },
		{ category: "pill", icon: "📜", count: "x3", name: "符篆", desc: "手绘符篆，激发后可释放一次法术，用后即毁。" },
		{ category: "pill", icon: "🔒", count: "x5", name: "封印格", desc: "尚未开启的储物格，修为达标后可解锁。", locked: true },
	]
  </script>
  <Panel name="GoodsPanel" class="grid-bg" anchor="full">
	<ScrollContainer name="GoodsScroll" anchor="full" margin="8">
	  <UIGrid name="GoodsGrid" columns="8" data="goods"
			 filter_key="category" filter_value_var="category"
			 h_separation="6" v_separation="6"
			 @s_click_item="_on_item_clicked"
			 size_flags_horizontal="expand_fill" size_flags_vertical="expand_fill">
		<!-- slot 模板：背包格子条目（构建期注入，duplicate 复制） -->
		<Gml src="bag_item.gml" />
	  </UIGrid>
	</ScrollContainer>
  </Panel>
</ui>
