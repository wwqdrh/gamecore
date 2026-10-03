<ui script="task_item.gd">
  <!-- 单个任务条目容器：作为 UIVList 的 slot 模板注入
	   script="task_item.gd"：控制器脚本自动挂载到条目根节点，
	   条目内 @pressed 信号就近绑定到该脚本（详见 task_item.gd 头注释）
	   {{key}} 为模板绑定，列表 update(data) 时按字段填充 -->
  <style>
	.item-bg {
	  background: #f6efdb;
	  border_color: #c9b98c;
	  border_width: 2;
	  border_radius: 10;
	}
	.icon-bg {
	  background: #e4d9b8;
	  border_radius: 8;
	}
	.item-title { color: #3c2f1c; }
	.item-desc { color: #8a7a58; }
	.item-progress { color: #5c4d2e; }
	.reward-num { color: #4a5a3a; }
	.go-btn {
	  background: #3f7d4e;
	  color: #ffffff;
	  border_radius: 8;
	}
  </style>
  <!-- 结构节点显式命名：allbind_signal 用 NodePath 绑定条目内部信号 -->
  <Panel name="ItemRoot" class="item-bg" custom_minimum_size="0,84">
	<MarginContainer name="ItemMargin" anchor="full" margin="12 8 12 8">
	  <HBoxContainer name="ItemRow">
		<!-- 图标占位：后期替换为 TextureRect/NinePatchRect 贴图 -->
		<Panel class="icon-bg" custom_minimum_size="56,56" size_flags_vertical="shrink_center">
		  <Label name="ItemIcon" text="{{icon}}" font_size="30" align="center" valign="center" anchor="full" />
		</Panel>
		<Control custom_minimum_size="10,0" />
		<VBoxContainer size_flags_horizontal="expand_fill" size_flags_vertical="shrink_center">
		  <Label name="ItemTitle" text="{{title}}" class="item-title" font_size="18" />
		  <Label name="ItemDesc" text="{{desc}}" class="item-desc" font_size="13" />
		</VBoxContainer>
		<Label name="ItemProgress" text="{{progress}}" class="item-progress" font_size="18" valign="center" size_flags_vertical="shrink_center" />
		<Control custom_minimum_size="14,0" />
		<Label name="Reward1" text="{{reward1}}" class="reward-num" font_size="15" valign="center" size_flags_vertical="shrink_center" />
		<Control custom_minimum_size="12,0" />
		<Label name="Reward2" text="{{reward2}}" class="reward-num" font_size="15" valign="center" size_flags_vertical="shrink_center" />
		<Control custom_minimum_size="14,0" />
		<!-- @pressed 声明信号绑定：就近解析到条目根挂载的 task_item.gd（见 <ui script>），
			 条目 duplicate 后由列表 bind_events 自动重连 -->
		<Button name="ItemBtn" text="{{btn_text}}" class="go-btn" custom_minimum_size="84,40" size_flags_vertical="shrink_center" @pressed="_on_task_action" />
	  </HBoxContainer>
	</MarginContainer>
  </Panel>
</ui>
