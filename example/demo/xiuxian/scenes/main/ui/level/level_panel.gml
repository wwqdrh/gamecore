<ui script="level_panel.gd">
  <!-- 修炼/功法面板：境界总览 + 纵向往上生长的功法树（GML <Tree> 组件）
	   树结构构建期静态声明（TreeNode 占位符契约，同 NavItem）：
	   根「道基」居底部，三系分支向上生长——
	   · 功法系（左）：吐纳诀→聚气术→周天功→气冲霄汉（灵力上限）
	   · 体修系（中）：炼皮→锻骨→易筋→金刚不坏（气血上限）
	   · 神识系（右）：凝神→静心→观想→神游太虚（悟性）
	   学习/解锁/存档逻辑见 level_panel.gd（<ui script> 就近绑定） -->
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
	.hint-text { color: #8a7d5e; font_size: 12; }
	.exp-track {
	  background: #d8ceb4;
	  border_color: #b7a477;
	  border_width: 1;
	  border_radius: 7;
	}
  </style>
  <Panel name="LevelPanel" class="window-bg" anchor="full" margin="2%"
	     mouse_filter="stop">
	<MarginContainer name="MainMargin" anchor="full" margin="16">
	  <VBoxContainer name="MainColumn" separation="10">
		<!-- 顶栏：境界 + 技能点 + 经验 -->
		<Panel name="TopBar" class="header-bg" custom_minimum_size="0,46">
		  <MarginContainer anchor="full" margin="10 14">
			<HBoxContainer name="TopRow" separation="16">
			  <Label name="RealmLabel" text="炼气一段" class="header-text" font_size="18"
			         valign="center" />
			  <Label name="PointsLabel" text="可用技能点 0" class="header-text" font_size="14"
			         valign="center" />
			  <Control size_flags_horizontal="expand_fill" />
			  <Label name="ExpLabel" text="经验 0/100" class="header-text" font_size="13"
			         valign="center" />
			</HBoxContainer>
		  </MarginContainer>
		</Panel>
		<!-- 经验条 -->
		<ProgressBar name="ExpBar" class="exp-track" custom_minimum_size="0,14"
		             percent_visible="false" />
		<!-- 功法树（纵向往上）：ScrollContainer 内自适应内容尺寸 -->
		<ScrollContainer name="TreeScroll" size_flags_vertical="expand_fill"
		                 horizontal="disabled">
		  <Tree name="GongfaTree" direction="up"
		        node_width="150" node_height="46" level_gap="42" sibling_gap="20"
		        font_size="14" line_color="#a08c5e" line_width="2"
		        node_bg="#f6f0de" node_border="#b7a477" node_text="#4a3f2a"
		        bg_done="#d9ecd2" border_done="#4f7d5c" text_done="#2e5638"
		        bg_locked="#e6e0d0" border_locked="#b0a88f" text_locked="#9a927c"
		        accent_color="#4a90d9" select_color="#c9873b">
			<TreeNode id="root" text="道基" desc="修行之始，气血自成（自带，无需领悟）。">
			  <!-- 功法系（左枝）：灵力上限 -->
			  <TreeNode id="gongfa1" text="吐纳诀" desc="入门吐纳法：灵力上限 +20。前置：道基">
				<TreeNode id="gongfa2" text="聚气术" desc="聚气成息：灵力上限 +40。前置：吐纳诀" locked="true">
				  <TreeNode id="gongfa3" text="周天功" desc="气行周天：灵力上限 +80。前置：聚气术" locked="true">
					<TreeNode id="gongfa4" text="气冲霄汉" desc="气贯长虹：灵力上限 +160。前置：周天功" locked="true" />
				  </TreeNode>
				</TreeNode>
			  </TreeNode>
			  <!-- 体修系（中枝）：气血上限 -->
			  <TreeNode id="body1" text="炼皮" desc="皮糙肉厚：气血上限 +50。前置：道基">
				<TreeNode id="body2" text="锻骨" desc="骨如精铁：气血上限 +100。前置：炼皮" locked="true">
				  <TreeNode id="body3" text="易筋" desc="筋长力沉：气血上限 +200。前置：锻骨" locked="true">
					<TreeNode id="body4" text="金刚不坏" desc="肉身成盾：气血上限 +400。前置：易筋" locked="true" />
				  </TreeNode>
				</TreeNode>
			  </TreeNode>
			  <!-- 神识系（右枝）：悟性 -->
			  <TreeNode id="soul1" text="凝神" desc="心无旁骛：悟性 +2。前置：道基">
				<TreeNode id="soul2" text="静心" desc="澄心如镜：悟性 +3。前置：凝神" locked="true">
				  <TreeNode id="soul3" text="观想" desc="内景观想：悟性 +5。前置：静心" locked="true">
					<TreeNode id="soul4" text="神游太虚" desc="神识出窍：悟性 +8。前置：观想" locked="true" />
				  </TreeNode>
				</TreeNode>
			  </TreeNode>
			</TreeNode>
		  </Tree>
		</ScrollContainer>
		<Label name="HintLabel" class="hint-text" align="center"
		       text="点击节点领悟功法（消耗 1 技能点）· 悬停查看详情 · 前置功法领悟后自动解锁下一层" />
	  </VBoxContainer>
	</MarginContainer>
  </Panel>
</ui>
