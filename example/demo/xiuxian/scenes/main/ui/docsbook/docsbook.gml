<ui script="docsbook.gd">
  <!-- 修仙手册（教程面板）：左树形菜单目录 + 右富文本正文。
       目录 = GML <TreeMenu>（多级折叠/自动缩进），正文 = GML <RichText>
       （.gdoc 文档解析：Markdown 子集 + BBCode + 图片）。
       文档源文件在 docs/ 目录（basics.gdoc / ability.gdoc），
       菜单选中 → docsbook.gd 按 id 加载对应 .gdoc 渲染 -->
  <style>
	.window-bg {
	  background: #f0e8d2;
	  border_color: #7a6a4a;
	  border_width: 3;
	  border_radius: 16;
	}
	.header-bg {
	  background: #5e8d7a;
	  border_color: #3f6b58;
	  border_width: 2;
	  border_radius: 10;
	}
	.header-text { color: #f2ecd9; }
	.side-bg {
	  background: #e8dfc4;
	  border_color: #c9b98c;
	  border_width: 1;
	  border_radius: 10;
	}
	.paper-bg {
	  background: #f7f2e2;
	  border_color: #c9b98c;
	  border_width: 1;
	  border_radius: 10;
	}
  </style>
  <Panel name="DocsPanel" class="window-bg" anchor="full" margin="2%"
	     mouse_filter="stop">
	<MarginContainer name="MainMargin" anchor="full" margin="16">
	  <VBoxContainer name="MainColumn" separation="10">
		<!-- 顶栏 -->
		<Panel name="TopBar" class="header-bg" custom_minimum_size="0,46">
		  <MarginContainer anchor="full" margin="10 14">
			<HBoxContainer name="TopRow" separation="16">
			  <Label name="TitleLabel" text="修仙手册" class="header-text" font_size="18"
			         valign="center" />
			  <Control size_flags_horizontal="expand_fill" />
			  <Label name="SubLabel" text="基础知识 · 能力体系" class="header-text"
			         font_size="13" valign="center" />
			</HBoxContainer>
		  </MarginContainer>
		</Panel>
		<!-- 左目录 + 右正文 -->
		<HBoxContainer name="BodyRow" size_flags_vertical="expand_fill" separation="12">
		  <Panel name="SideBar" class="side-bg" custom_minimum_size="250,0">
			<MarginContainer name="SideMargin" anchor="full" margin="8">
			  <ScrollContainer name="MenuScroll" horizontal="disabled">
				<TreeMenu name="DocMenu" indent_width="18" row_height="36" row_gap="2"
				          font_size="15"
				          item_bg="#00000000" item_text="#4a3f2a"
				          item_hover="#c9b98c40"
				          selected_bg="#d8cba0" selected_text="#2e2415"
				          @s_menu_selected="_on_menu_selected">
				  <MenuNode id="basics" text="基础知识">
					<MenuNode id="ability" text="能力体系" />
				  </MenuNode>
				</TreeMenu>
			  </ScrollContainer>
			</MarginContainer>
		  </Panel>
		  <Panel name="DocArea" class="paper-bg" size_flags_horizontal="expand_fill">
			<MarginContainer name="DocMargin" anchor="full" margin="14 20">
			  <RichText name="DocView"
			            src="res://example/demo/xiuxian/scenes/main/ui/docsbook/docs/basics.gdoc"
			            font_size="15"
			            text_color="#3a2f1e" heading_color="#8a4a1a"
			            scroll_active="true" />
			</MarginContainer>
		  </Panel>
		</HBoxContainer>
	  </VBoxContainer>
	</MarginContainer>
  </Panel>
</ui>
