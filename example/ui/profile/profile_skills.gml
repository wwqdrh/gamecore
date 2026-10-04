<ui>
  <!-- 右栏：功法列表（UIVList + skill_item 条目模板，构建期数据驱动填充）
	   skills 数据也可换成 data="bean:xxx:skills" 走运行时响应式绑定 -->
  <style>
	.card-bg {
	  background: #f6f0de;
	  border_color: #c9b98c;
	  border_width: 2;
	  border_radius: 10;
	}
	.list-header {
	  background: #5e8d7a;
	  border_color: #3f6b58;
	  border_width: 2;
	  border_radius: 8;
	}
	.header-text { color: #f2ecd9; }
  </style>
  <script>
    // 功法数据：locked=true 的条目由 skill_item.gd 置灰并禁用按钮
    var skills = [
      { icon: "🌀", title: "吐纳诀", level: "Lv.4", btn_text: "升级", locked: false },
      { icon: "⚔️", title: "青木剑诀", level: "Lv.2", btn_text: "升级", locked: false },
      { icon: "🌪️", title: "御风术", level: "未习得", btn_text: "🔒", locked: true },
    ]
  </script>
  <Panel name="SkillsPanel" class="card-bg">
	<VBoxContainer name="SkillsColumn" anchor="full" margin="10">
	  <Panel name="SkillsHeader" class="list-header" custom_minimum_size="0,36">
		<Label name="SkillsHeaderText" text="功法列表" class="header-text" font_size="17"
		       align="center" valign="center" anchor="full" />
	  </Panel>
	  <Control custom_minimum_size="0,8" />
	  <ScrollContainer name="SkillsScroll" size_flags_vertical="expand_fill" horizontal="disabled">
		<UIVList name="SkillList" data="skills" size_flags_horizontal="expand_fill">
		  <Gml src="skill_item.gml" />
		</UIVList>
	  </ScrollContainer>
	</VBoxContainer>
  </Panel>
</ui>
