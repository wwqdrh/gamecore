# Stage1 M3 验收：对话 timeline 静态 lint + DialogBox 新命令动态验证
#
# 静态 lint（全部 10 份 timeline）：
#   L1 goto 目标 stage 必须存在
#   L2 gate 引用的 flag 必须可溯源：task_<id>_accepted/_done 的 id 在任务表；
#      其余 flag 必须被某处 set_flag 过，或属于代码置位白名单
#   L3 危险变更模式：行上 @cmd（set_flag/clear_flag/task_accept/task_complete/
#      task_advance/item_give）改变的 flag 若参与本段 gate 表达式 → FAIL
#      （行推进时 gate 失配 → 对话断线，变更必须写进选项动作）
#   L4 选项动作链语法：';' 后续命令不得带 @ 前缀（exec_response 不识别）
#
# 动态：item_give 入包（int 与 String 数量参）、task_advance 未接取无效果/
#   接取后推进、task_status 查询、GdDialogue 解析 elder timeline 的 stage 表。
#
# 运行：godot --headless -s res://test/check_stage1_dialog.gd
extends SceneTree

const NPC_DIR := "res://example/demo/xiuxian/role/npc/"
const TIMELINES := [
	"elder/elder_timeline.txt", "xiao_master/master_timeline.txt",
	"xiao_maid/maid_timeline.txt", "xiao_lady/lady_timeline.txt",
	"storyteller/story_timeline.txt", "disciple/disciple_timeline.txt",
	"merchant/merchant_timeline.txt", "herb_shop/herb_timeline.txt",
	"blacksmith/smith_timeline.txt", "arena_master/arena_timeline.txt",
]
## 代码/回调置位、不在 timeline 内 set_flag 的 flag 白名单
const CODE_FLAGS := ["rank_dou_zhe", "day7_arrived", "tournament_won", "match_started"]

var _ok := true


func _check(cond: bool, msg: String) -> void:
	if cond:
		print("[Stage1Dialog] OK ", msg)
	else:
		_ok = false
		printerr("[Stage1Dialog] FAIL ", msg)


func _init() -> void:
	call_deferred("_run")


func _run() -> void:
	_lint_all()
	await process_frame
	_dynamic()
	print("[Stage1Dialog] RESULT: %s" % ("PASS" if _ok else "FAIL"))
	quit(0 if _ok else 1)


# ----------------------------------------------------------------- 静态 lint

func _lint_all() -> void:
	var task_ids := {}
	for t in XiuTaskTable.get_tasks().values():
		task_ids[str(t.get("id", ""))] = true
	var defined_flags := {}   # 全部 timeline 中 set_flag 过的 flag
	var parsed := {}          # 文件名 -> {stage_name: {gate: Array, lines: Array}}

	for rel in TIMELINES:
		var f := FileAccess.open(NPC_DIR + rel, FileAccess.READ)
		_check(f != null, "打开 %s" % rel)
		if f == null:
			continue
		var stages := {}
		var cur := ""
		for line in f.get_as_text().split("\n"):
			var s := line.strip_edges()
			if s.is_empty() or s.begins_with("#"):
				continue
			if s.begins_with("[") and "]" in s:
				var head := s.substr(1, s.find("]") - 1)
				var at := head.find("@")
				cur = head.substr(0, at).strip_edges() if at >= 0 else head.strip_edges()
				var gate_expr := head.substr(at + 1) if at >= 0 else ""
				var gates := []
				for g in gate_expr.split(";", false):
					var t := g.strip_edges()
					if not t.is_empty():
						gates.append(t)
				stages[cur] = {"gate": gates, "lines": []}
			elif cur != "":
				stages[cur]["lines"].append(s)
		parsed[rel] = stages
		f.close()

	# 先收集全部 set_flag 的 flag（跨文件可溯源）
	for rel in parsed:
		for sname in parsed[rel]:
			for line in parsed[rel][sname]["lines"]:
				for fl in _extract_cmds(line, "set_flag"):
					defined_flags[fl] = true

	for rel in parsed:
		var stages: Dictionary = parsed[rel]
		# L1 goto 目标
		for sname in stages:
			for line in stages[sname]["lines"]:
				for m in _regex_all(line, "goto:([A-Za-z0-9_#]+)"):
					if m.begins_with("#"):
						continue
					_check(stages.has(m), "%s [%s] goto 目标存在: %s" % [rel, sname, m])
		# L2 gate flag 溯源 + L3 危险变更
		for sname in stages:
			var gate_flags := []
			for g in stages[sname]["gate"]:
				var t: String = g.strip_edges().lstrip("!")
				if t.begins_with("task_"):
					# 任务 flag 三态：_accepted / _done / _step<N>（advance 置）
					var tid: String = t.trim_prefix("task_")
					var re := RegEx.new()
					re.compile("_(accepted|done|step\\d+)$")
					tid = re.sub(tid, "", true)
					_check(task_ids.has(tid),
						"%s [%s] gate 引用任务存在: %s" % [rel, sname, tid])
				elif ":" in t:
					pass  # method:args 形式，运行期对 control 求值，不静态查
				else:
					gate_flags.append(t)
					_check(defined_flags.has(t) or CODE_FLAGS.has(t),
						"%s [%s] gate flag 可溯源: %s" % [rel, sname, t])
			for line in stages[sname]["lines"]:
				if not line.begins_with("@"):
					continue
				var changed := _changed_flags(line)
				for cf in changed:
					_check(not gate_flags.has(cf),
						"%s [%s] 行上变更 %s 不参与本段 gate（须写进选项动作）"
						% [rel, sname, cf])
		# L4 选项动作链 @ 前缀
		for sname in stages:
			for line in stages[sname]["lines"]:
				if not line.begins_with("-") or not line.contains("@"):
					continue
				var act: String = line.substr(line.rfind("@") + 1)
				var parts: PackedStringArray = act.split(";", false)
				for i in parts.size():
					if i > 0 and parts[i].strip_edges().begins_with("@"):
						_check(false, "%s [%s] 链式动作后续命令带 @（不识别）: %s"
							% [rel, sname, parts[i]])
	_check(parsed.size() == TIMELINES.size(), "lint 覆盖 %d 份 timeline" % TIMELINES.size())


## 提取行内所有 @cmd:args（行函数行）或选项动作里的命令
func _extract_cmds(line: String, fn_name: String) -> Array:
	var out := []
	for m in _regex_all(line, "%s:([A-Za-z0-9_,]+)" % fn_name):
		out.append(m)
	return out


## 行函数 @cmd 改变的 flag 集合（task_accept→accepted / task_complete→done）
func _changed_flags(line: String) -> Array:
	var out := []
	for fl in _extract_cmds(line, "set_flag"):
		out.append(fl)
	for fl in _extract_cmds(line, "clear_flag"):
		out.append(fl)
	for tid in _extract_cmds(line, "task_accept"):
		out.append("task_%s_accepted" % tid)
	for tid in _extract_cmds(line, "task_complete"):
		out.append("task_%s_done" % tid)
	for tid in _extract_cmds(line, "task_advance"):
		# 步级 flag 上界取不到 steps，按宽上界枚举（仅行函数场景，保守覆盖）
		for i in range(1, 10):
			out.append("task_%s_step%d" % [tid, i])
	return out


func _regex_all(text: String, pattern: String) -> Array:
	var re := RegEx.new()
	if re.compile(pattern) != OK:
		return []
	var out := []
	for m in re.search_all(text):
		out.append(m.get_string(1))
	return out


# ----------------------------------------------------------------- 动态验证

func _dynamic() -> void:
	# 1. GdDialogue 解析 elder timeline（stage 表完整性）
	var dlg := GdDialogue.new()
	root.add_child(dlg)
	dlg.set_timeline_path(NPC_DIR + "elder/elder_timeline.txt")
	var names := dlg.all_stages()
	var want := ["elder_report", "elder_daily", "elder_talk",
		"elder_guide", "elder_market", "elder_end"]
	var got := {}
	for n in names:
		got[str(n)] = true
	var all_in := true
	for w in want:
		if not got.has(w):
			all_in = false
	_check(all_in, "elder timeline 解析出全部 %d 个 stage（got %d）" % [want.size(), names.size()])
	dlg.queue_free()

	# 2. item_give：int 与 String 数量参（exec_response 以字符串传参）
	var db: Node = load("res://example/demo/xiuxian/ui/dialog/dialog_box.gd").new()
	var before := XiuItemState.ins().get_count("break_qizhe")
	db.item_give("break_qizhe", 2)
	var mid := XiuItemState.ins().get_count("break_qizhe")
	db.item_give("break_qizhe", "3")
	var after := XiuItemState.ins().get_count("break_qizhe")
	_check(mid - before == 2, "item_give(int) 入包 +2（got +%d）" % (mid - before))
	_check(after - mid == 3, "item_give(String) 入包 +3（got +%d）" % (after - mid))

	# 3. task_advance：未接取无效果，接取后推进
	var ts := XiuTaskState.ins()
	db.task_advance("main_105")
	_check(ts.get_task_status("main_105") != ts.STATUS_ACCEPTED,
		"task_advance 对未接取任务无效果")
	ts.reset_demo()
	ts.unlock_task("main_105")
	ts.accept_task("main_105")
	var step0 := int(ts.get_progress("main_105").get("step", 0))
	db.task_advance("main_105")
	var step1 := int(ts.get_progress("main_105").get("step", 1))
	_check(step1 == step0 + 1, "task_advance 已接取任务推进一步（%d→%d）" % [step0, step1])

	# 4. task_status 门控查询
	_check(db.task_status("main_105", "accepted"), "task_status 查询 accepted")
	_check(not db.task_status("main_105", "completed"), "task_status 状态不符返回 false")
	ts.reset_demo()
	db.queue_free()
