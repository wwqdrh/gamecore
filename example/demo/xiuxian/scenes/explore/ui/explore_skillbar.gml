<ui script="explore_skillbar.gd">
  <!-- 秘境技能栏（自 mainhud 迁移，仅探索场景显示）：6 格技能（UIHList 数据驱动 +
	   冷却态）+ 灵气经验条。数据绑定链路：<script> 定义技能数据（数据自带
	   key/icon/cd 展示字段），UIHList data="skills" 构建期填充；
	   @s_click_item 回调本文件脚本，点选只上报 GdState（mainhud.skill 键名保留，
	   玩家侧消费方 watch 同名键，跨场景兼容） -->
  <style>
	.xp-bg {
	  background: #2e4a3a;
	  border_color: #e8b93e;
	  border_width: 2;
	  border_radius: 12;
	}
	.xp-track {
	  background: #243c2e;
	  border_color: #56705c;
	  border_width: 1;
	  border_radius: 6;
	}
	.xp-fill {
	  background: #4fc3b8;
	  border_radius: 6;
	}
	.xp-text { color: #f2ead2; font_size: 13; }
  </style>
  <script>
	// 技能栏数据（6 格）：key 键位角标 / icon 图标 / cd 冷却文本（空串 = 就绪）。
	// key 对应 player.gd SKILL_PATTERNS（弹幕阵型）：1 灵光速射 / 2 玄风扇煞 /
	// 3 周天星环 / 4 碧波灵刃 / 5 追魂符引 / 6 万剑归宗
	var skills = [
		{ key: "1", icon: "⚡", cd: "" },
		{ key: "2", icon: "🌀", cd: "" },
		{ key: "3", icon: "🌙", cd: "2.5s" },
		{ key: "4", icon: "🍃", cd: "1.6s" },
		{ key: "5", icon: "🧘", cd: "2s" },
		{ key: "6", icon: "🗡️", cd: "6s" },
	]
  </script>
  <VBoxContainer name="SkillBar" separation="8">
	<UIHList name="SkillList" data="skills" h_separation="10"
	         @s_click_item="_on_skill_clicked">
	  <!-- slot 模板：技能格条目（构建期注入，duplicate 复制） -->
	  <Gml src="skill_item.gml" />
	</UIHList>
	<!-- 灵气经验条：由 explore_skillbar.gd watch 状态层经验驱动 -->
	<Panel name="XpBar" class="xp-bg" custom_minimum_size="500,26">
	  <HBoxContainer anchor="full" margin="12 4 12 4" separation="8">
		<Label text="💠" valign="center" font_size="12" />
		<Label class="xp-text" text="灵气经验" valign="center" />
		<Panel name="XpTrack" class="xp-track" custom_minimum_size="0,12"
		       size_flags_horizontal="expand_fill" size_flags_vertical="shrink_center">
		  <Panel name="XpFill" class="xp-fill" custom_minimum_size="0,12"
		         anchor="left_wide" />
		</Panel>
		<Label name="XpText" class="xp-text" text="0/100" valign="center" />
	  </HBoxContainer>
	</Panel>
  </VBoxContainer>
</ui>
