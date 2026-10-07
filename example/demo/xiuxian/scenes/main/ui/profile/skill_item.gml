<ui script="skill_item.gd">
  <!-- 功法条目模板：作为 profile_skills.gml 中 UIVList 的 slot 模板
	   <ui script> 把条目控制器挂到条目根节点（duplicate 随条目复制），
	   SkillBtn @pressed 就近绑定到本条目自己的脚本实例；
	   locked 契约字段注入时由脚本置灰整条目并禁用按钮 -->
  <style>
	.skill-bg {
	  background: #efe6cd;
	  border_color: #c9b98c;
	  border_width: 2;
	  border_radius: 10;
	}
	.skill-icon {
	  background: #4a7a68;
	  border_color: #35594b;
	  border_width: 2;
	  border_radius: 8;
	}
	.skill-title { color: #3c2f1c; }
	.skill-level { color: #8a7a58; }
	.gold-btn {
	  background: #e8b93e;
	  color: #4a3410;
	  border_color: #8a6a1e;
	  border_width: 2;
	  border_radius: 10;
	}
  </style>
  <Panel name="SkillItemRoot" class="skill-bg" custom_minimum_size="0,72">
	<MarginContainer name="SkillMargin" anchor="full" margin="8">
	  <HBoxContainer name="SkillRow">
		<Panel name="SkillIconBg" class="skill-icon" custom_minimum_size="52,52" size_flags_vertical="shrink_center">
		  <Label name="SkillIcon" text="{{icon}}" font_size="24" align="center" valign="center" anchor="full" />
		</Panel>
		<Control custom_minimum_size="10,0" />
		<VBoxContainer name="SkillInfo" size_flags_horizontal="expand_fill" size_flags_vertical="shrink_center">
		  <Label name="SkillTitle" text="{{title}}" class="skill-title" font_size="17" />
		  <Label name="SkillLevel" text="{{level}}" class="skill-level" font_size="13" />
		</VBoxContainer>
		<Control custom_minimum_size="8,0" />
		<Button name="SkillBtn" text="{{btn_text}}" class="gold-btn"
		        custom_minimum_size="76,40" size_flags_vertical="shrink_center"
		        @pressed="_on_skill_action" />
	  </HBoxContainer>
	</MarginContainer>
  </Panel>
</ui>
