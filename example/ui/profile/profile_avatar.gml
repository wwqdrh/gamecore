<ui>
  <!-- 左栏：角色立绘占位 + 竖排"修仙"标签 + 五行灵根列表
	   立绘为 ColorRect 占位，正式版替换 TextureRect 贴图；
	   五行灵根用 <script> 数据 + UIVList 数据驱动（构建期直接填充） -->
  <style>
	.card-bg {
	  background: #f6f0de;
	  border_color: #c9b98c;
	  border_width: 2;
	  border_radius: 10;
	}
	.portrait-bg {
	  background: #d8e4d8;
	  border_color: #b0c2b0;
	  border_width: 2;
	  border_radius: 8;
	}
	.xian-badge {
	  background: #a8443a;
	  color: #f2ecd9;
	  border_radius: 6;
	}
	.section-title { color: #6b5a3e; }
  </style>
  <script>
    // 五行灵根数据（icon/名称/当前值/上限），构建期直接驱动 SpiritRootList
    var spirit_roots = [
      { icon: "🟡", element: "金", value: 28, max: 100 },
      { icon: "🟢", element: "木", value: 32, max: 100 },
      { icon: "🔵", element: "水", value: 24, max: 100 },
      { icon: "🔴", element: "火", value: 18, max: 100 },
      { icon: "🟤", element: "土", value: 16, max: 100 },
    ]
  </script>
  <Panel name="AvatarPanel" class="card-bg">
	<VBoxContainer name="AvatarColumn" anchor="full" margin="10">
	  <Panel name="PortraitArea" class="portrait-bg" size_flags_vertical="expand_fill">
		<Label name="PortraitPlaceholder" text="【立绘】" font_size="20" align="center" valign="center" anchor="full" />
		<Panel name="XianBadge" class="xian-badge" custom_minimum_size="28,64"
		       anchor="top_left" margin="8 8 0 0">
		  <VBoxContainer name="XianText" anchor="full">
			<Label text="修" font_size="14" align="center" valign="center" size_flags_vertical="expand_fill" />
			<Label text="仙" font_size="14" align="center" valign="center" size_flags_vertical="expand_fill" />
		  </VBoxContainer>
		</Panel>
	  </Panel>
	  <Control custom_minimum_size="0,8" />
	  <Label name="RootSectionTitle" text="五行灵根" class="section-title" font_size="17" align="center" />
	  <Control custom_minimum_size="0,4" />
	  <UIVList name="SpiritRootList" data="spirit_roots" size_flags_horizontal="expand_fill">
		<Gml src="spirit_root_item.gml" />
	  </UIVList>
	</VBoxContainer>
  </Panel>
</ui>
