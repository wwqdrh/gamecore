#include "flow.h"

namespace gamedialog {
void StartFlow::execute(Timeline &timeline) {
  // 实现回到开头的逻辑
  timeline.goto_begin();
}
void EndFlow::execute(Timeline &timeline) { timeline.goto_end(); }
void SkipFlow::execute(Timeline &timeline) {
  timeline.skip_stage_count(skipCount);
}
void GotoFlow::execute(Timeline &timeline) { timeline.goto_stage(targetName); }

bool GotoFlow::hasNext(const Timeline &timeline) const {
  return timeline.has_stage(targetName);
}

std::shared_ptr<ControlFlow>
ControlFlowFactory::createFromString(const std::string &command) {
  // 解析命令字符串
  if (command == ":start") {
    return std::make_shared<StartFlow>();
  } else if (command == ":end") {
    return std::make_shared<EndFlow>();
  } else if (command.substr(0, 6) == ":skip:") {
    int count = std::stoi(command.substr(6));
    return std::make_shared<SkipFlow>(count);
  } else if (command.substr(0, 6) == ":goto:") {
    std::string target = command.substr(6);
    return std::make_shared<GotoFlow>(target);
  } else {
    return nullptr;
  }
}
} // namespace gamedialog