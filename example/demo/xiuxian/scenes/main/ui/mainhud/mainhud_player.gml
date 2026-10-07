<ui>
  <!-- 左上玩家徽章：圆形头像 + 境界卷轴横幅（最高境界角标）+ 等级行。
	   头像 @pressed="show:ProfileModal"：角色面板在 ui/profile/ 目录，
	   内部动作经统一 UI 管理层按 ui_id 跨组件触发（零文件引用）。
	   等级/境界/经验由 mainhud.gd 绑定状态层 XiuCharacterState
	   （bean: xiuxian_character）watch 驱动（锚点节点已命名） -->
  <style>
	.avatar-ring {
	  background: #2e4a3a;
	  border_color: #e8b93e;
	  border_width: 3;
	  border_radius: 44;
	}
	.realm-banner {
	  background: #e8d9a8;
	  border_color: #8a6a1e;
	  border_width: 2;
	  border_radius: 8;
	}
	.realm-text { color: #6b4a1e; font_size: 20; }
	.realm-tag {
	  color: #f2ead2;
	  font_size: 11;
	  background: #8a6a1e;
	  border_radius: 4;
	}
	.lv-text { color: #e8b93e; font_size: 14; }
	.lv-track {
	  background: #243c2e;
	  border_radius: 5;
	}
	.lv-fill {
	  background: #5fae6a;
	  border_radius: 5;
	}
	.lv-pct { color: #cfe3c8; font_size: 12; }
	.avatar-icon { font_size: 42; }
  </style>
  <HBoxContainer name="PlayerBadge" separation="10">
	<!-- 圆形头像（点击弹出角色信息 Modal，组合根 main.gml 装配） -->
	<Panel name="AvatarRing" class="avatar-ring" custom_minimum_size="88,88"
	       mouse_default_cursor_shape="pointing_hand" @pressed="show:ProfileModal">
	  <Label name="AvatarIcon" class="avatar-icon" text="🧙" anchor="full"
	         align="center" valign="center" />
	</Panel>
	<!-- 境界卷轴 + 等级行 -->
	<VBoxContainer name="PlayerInfo" separation="6" size_flags_vertical="shrink_center">
	  <Panel class="realm-banner" custom_minimum_size="210,42">
		<HBoxContainer anchor="full" margin="12 6 8 6" separation="8">
		  <Label name="RealmText" class="realm-text" text="练气一层" valign="center" />
		  <Control size_flags_horizontal="expand_fill" />
		  <Label class="realm-tag" text="最高境界" valign="center" />
		</HBoxContainer>
	  </Panel>
	  <HBoxContainer separation="8">
		<Label name="LevelText" class="lv-text" text="Lv.1" valign="center" />
		<Panel class="lv-track" custom_minimum_size="90,10" size_flags_vertical="shrink_center">
		  <Panel name="LevelFill" class="lv-fill" custom_minimum_size="0,10" anchor="left_wide" />
		</Panel>
		<Label name="LevelPct" class="lv-pct" text="0%" valign="center" />
	  </HBoxContainer>
	</VBoxContainer>
  </HBoxContainer>
</ui>
