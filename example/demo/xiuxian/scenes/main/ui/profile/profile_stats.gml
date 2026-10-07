<ui>
  <!-- 底部战斗属性条：攻击/防御/气血/灵力 四组（icon + 名称 + 数值），expand 均分 -->
  <style>
	.stats-bg {
	  background: #e9dfc4;
	  border_color: #b8a678;
	  border_width: 2;
	  border_radius: 12;
	}
	.stat-name { color: #6b5a3e; }
	.stat-value { color: #3c2f1c; }
  </style>
  <Panel name="StatsBar" class="stats-bg" custom_minimum_size="0,56">
	<HBoxContainer name="StatsRow" anchor="full" margin="8">
	  <HBoxContainer name="StatAttack">
		<Label text="⚔️" font_size="17" valign="center" />
		<Control custom_minimum_size="6,0" />
		<Label text="攻击" class="stat-name" font_size="16" valign="center" />
		<Control custom_minimum_size="6,0" />
		<Label name="AttackValue" text="128" class="stat-value" font_size="17" valign="center" />
	  </HBoxContainer>
	  <Control size_flags_horizontal="expand_fill" />
	  <HBoxContainer name="StatDefense">
		<Label text="🛡️" font_size="17" valign="center" />
		<Control custom_minimum_size="6,0" />
		<Label text="防御" class="stat-name" font_size="16" valign="center" />
		<Control custom_minimum_size="6,0" />
		<Label name="DefenseValue" text="86" class="stat-value" font_size="17" valign="center" />
	  </HBoxContainer>
	  <Control size_flags_horizontal="expand_fill" />
	  <HBoxContainer name="StatHealth">
		<Label text="❤️" font_size="17" valign="center" />
		<Control custom_minimum_size="6,0" />
		<Label text="气血" class="stat-name" font_size="16" valign="center" />
		<Control custom_minimum_size="6,0" />
		<Label name="HealthValue" text="756" class="stat-value" font_size="17" valign="center" />
	  </HBoxContainer>
	  <Control size_flags_horizontal="expand_fill" />
	  <HBoxContainer name="StatMana">
		<Label text="💧" font_size="17" valign="center" />
		<Control custom_minimum_size="6,0" />
		<Label text="灵力" class="stat-name" font_size="16" valign="center" />
		<Control custom_minimum_size="6,0" />
		<Label name="ManaValue" text="214" class="stat-value" font_size="17" valign="center" />
	  </HBoxContainer>
	</HBoxContainer>
  </Panel>
</ui>
