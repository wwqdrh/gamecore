#pragma once

#include "stage.h"
#include "timeline.h"
#include <string>

namespace gamedialog {
class Timeline;
class DiaStage;
// 基础控制流类
class ControlFlow {
public:
  std::string stage_name;

public:
  virtual ~ControlFlow() = default;
  virtual void execute(Timeline &timeline) = 0;
  virtual bool hasNext(const Timeline &timeline) const = 0;
  virtual std::string getName() const = 0;

public:
  void set_stage_name(const std::string &name) { stage_name = name; }
  const std::string &get_stage_name() const { return stage_name; }
};

// Start分支 - 回到开头
class StartFlow : public ControlFlow {
public:
  void execute(Timeline &timeline) override;

  std::string getName() const override { return "start"; }
  bool hasNext(const Timeline &timeline) const override { return true; }
};

// End分支 - 退出
class EndFlow : public ControlFlow {
public:
  void execute(Timeline &timeline) override;

  std::string getName() const override { return "end"; }
  bool hasNext(const Timeline &timeline) const override { return false; }
};

// Skip分支 - 跳过n个stage
class SkipFlow : public ControlFlow {
private:
  int skipCount;

public:
  explicit SkipFlow(int count) : skipCount(count) {}

  void execute(Timeline &timeline) override;

  std::string getName() const override { return "skip"; }
  bool hasNext(const Timeline &timeline) const override { return true; }

  int getSkipCount() const { return skipCount; }
};

// Goto分支 - 跳转到指定stage
class GotoFlow : public ControlFlow {
private:
  std::string targetName;

public:
  explicit GotoFlow(const std::string &name) : targetName(name) {}

  void execute(Timeline &timeline) override;

  std::string getName() const override { return "goto"; }
  bool hasNext(const Timeline &timeline) const override;
  std::string getTargetName() const { return targetName; }
};

// 控制流工厂类
class ControlFlowFactory {
public:
  static std::shared_ptr<ControlFlow>
  createFromString(const std::string &command);
};
} // namespace gamedialog