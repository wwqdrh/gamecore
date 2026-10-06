<ui>
  <!-- 左上玩家徽章：圆形头像 + 境界卷轴横幅（最高境界角标）+ 等级行。
	   无脚本：静态展示区块（等级/境界接 GdBean 后可改 {{}} 模板） -->
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
	<!-- 圆形头像 -->
	<Panel name="AvatarRing" class="avatar-ring" custom_minimum_size="88,88">
	  <Label name="AvatarIcon" class="avatar-icon" text="🧙" anchor="full"
	         align="center" valign="center" />
	</Panel>
	<!-- 境界卷轴 + 等级行 -->
	<VBoxContainer name="PlayerInfo" separation="6" size_flags_vertical="shrink_center">
	  <Panel class="realm-banner" custom_minimum_size="210,42">
		<HBoxContainer anchor="full" margin="12 6 8 6" separation="8">
		  <Label class="realm-text" text="炼气三层" valign="center" />
		  <Control size_flags_horizontal="expand_fill" />
		  <Label class="realm-tag" text="最高境界" valign="center" />
		</HBoxContainer>
	  </Panel>
	  <HBoxContainer separation="8">
		<Label class="lv-text" text="Lv.12" valign="center" />
		<Panel class="lv-track" custom_minimum_size="90,10" size_flags_vertical="shrink_center">
		  <Panel class="lv-fill" custom_minimum_size="30,10" anchor="left_wide" />
		</Panel>
		<Label class="lv-pct" text="22" valign="center" />
	  </HBoxContainer>
	</VBoxContainer>
  </HBoxContainer>
</ui>
