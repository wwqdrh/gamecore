<ui>
  <!-- 中央境界突破区：当前/下一境界 + 62% 进度圆环 + 突破按钮 + 突破材料
	   圆环为 Panel 嵌套占位（外圈金环 + 内圈玉底 + 中央百分比），
	   正式版替换为自定义环形进度控件或纹理帧动画 -->
  <style>
	.card-bg {
	  background: #f6f0de;
	  border_color: #c9b98c;
	  border_width: 2;
	  border_radius: 10;
	}
	.realm-now { color: #3c2f1c; }
	.realm-arrow { color: #a8443a; }
	.realm-next { color: #5e8d7a; }
	.ring-outer {
	  background: #d9c892;
	  border_color: #b8a06a;
	  border_width: 3;
	  border_radius: 90;
	}
	.ring-inner {
	  background: #e9f0e6;
	  border_color: #5e9c86;
	  border_width: 4;
	  border_radius: 74;
	}
	.ring-percent { color: #3f6b58; }
	.gold-btn {
	  background: #e8b93e;
	  color: #4a3410;
	  border_color: #8a6a1e;
	  border_width: 2;
	  border_radius: 12;
	}
	.item-chip {
	  background: #efe4c4;
	  border_color: #c9b98c;
	  border_width: 2;
	  border_radius: 16;
	}
	.chip-text { color: #6b5a3e; }
  </style>
  <Panel name="RealmPanel" class="card-bg">
	<VBoxContainer name="RealmColumn" anchor="full" margin="14">
	  <CenterContainer name="RealmTitleRow">
		<HBoxContainer>
		  <Label name="RealmNow" text="炼气三层" class="realm-now" font_size="20" valign="center" />
		  <Control custom_minimum_size="10,0" />
		  <Label name="RealmArrow" text="→" class="realm-arrow" font_size="20" valign="center" />
		  <Control custom_minimum_size="10,0" />
		  <Label name="RealmNext" text="炼气四层" class="realm-next" font_size="20" valign="center" />
		</HBoxContainer>
	  </CenterContainer>
	  <Control custom_minimum_size="0,8" />
	  <CenterContainer name="RingRow" size_flags_vertical="expand_fill">
		<Panel name="ProgressRing" class="ring-outer" custom_minimum_size="160,160">
		  <Panel name="RingInner" class="ring-inner" anchor="full" margin="14">
			<Label name="RealmProgress" text="62%" class="ring-percent" font_size="34"
			       align="center" valign="center" anchor="full" />
		  </Panel>
		</Panel>
	  </CenterContainer>
	  <Control custom_minimum_size="0,10" />
	  <CenterContainer name="BreakthroughRow">
		<Button name="BreakthroughBtn" text="突  破" class="gold-btn"
		        custom_minimum_size="180,52" @pressed="_on_breakthrough_pressed" />
	  </CenterContainer>
	  <Control custom_minimum_size="0,8" />
	  <CenterContainer name="MaterialRow">
		<Panel name="MaterialChip" class="item-chip" custom_minimum_size="130,40">
		  <HBoxContainer name="MaterialChipRow" anchor="full" margin="10 0 10 0">
			<Label name="MaterialIcon" text="🟢" font_size="16" valign="center" />
			<Control custom_minimum_size="6,0" />
			<Label name="MaterialText" text="聚气丹 x5" class="chip-text" font_size="15" valign="center" />
		  </HBoxContainer>
		</Panel>
	  </CenterContainer>
	</VBoxContainer>
  </Panel>
</ui>
