#include "stage.h"
#include <gtest/gtest.h>

#include "stage.h"
#include "timeline.h"

using namespace gamedialog;

TEST(StageTest, TestParse) {
  gamedialog::DiaStage stage(R"(
[stage1]
(John,Mary)
Hello there!
-Yes: next_scene
-No: end_scene
Yes, Hello!
    )");

  ASSERT_EQ(stage.get_stage_name(), "stage1");
  ASSERT_EQ(stage.get_line_size(), 2);
}

TEST(StageTest, TestStageExprAll) {
  // 各种控制分支
  // :start 回到开头
  // :end 表示退出
  // :skip:1 表示跳过n个stage
  // :goto:name 跳转到指定的stage
  Timeline parser(R"(
[stage1]
(John)
Hello there!
:skip:2

[next_scene]
(Mary)
answer yes!!
:end

[end_scene]
(John)
answer no!!
:start

[goto_scene]
(John)
answer no!!
:goto:noexist
)");

  auto stages = parser.all_stages();
  ASSERT_EQ(4, stages.size());
  ASSERT_EQ(parser.next()->get_name(), "John");
  ASSERT_EQ(parser.current_stage(), "stage1");
  ASSERT_TRUE(parser.has_next());

  // skip:2 + next
  auto cur = parser.next();
  ASSERT_TRUE(parser.has_next());
  ASSERT_EQ(parser.current_stage(), "end_scene");
  ASSERT_EQ(cur->get_text(), "answer no!!");
  ASSERT_TRUE(parser.has_next());

  // :start + next
  cur = parser.next();
  ASSERT_EQ(parser.current_stage(), "stage1");
  ASSERT_EQ(cur->get_text(), "Hello there!");
  ASSERT_EQ(parser.current_stage(), "stage1");

  // 测试end
  parser.goto_stage("next_scene");
  ASSERT_EQ(parser.current_stage(), "next_scene");
  ASSERT_TRUE(parser.has_next());
  parser.next();
  ASSERT_FALSE(parser.has_next()); // 当前是:end标签，那么就没有下一个元素了

  // 测试goto
  parser.goto_stage("goto_scene");
  ASSERT_EQ(parser.current_stage(), "goto_scene");
  ASSERT_TRUE(parser.has_next());
  parser.next();
  ASSERT_FALSE(parser.has_next()); // 当前是:end标签，那么就没有下一个元素了
}

TEST(StageTest, TestGotoExpr) {
  Timeline parser(R"(
[stage1]
(John)
Hello there!
:goto:stage2

[stage2]
(Mary)
answer yes!!
)");

  ASSERT_EQ(2, parser.all_stages().size());
  ASSERT_EQ(parser.current_stage(), "stage1");
  parser.next();

  // goto + next
  auto cur = parser.next();
  ASSERT_EQ(parser.current_stage(), "stage2");
  ASSERT_EQ(cur->get_name(), "Mary");
  ASSERT_EQ(cur->get_text(), "answer yes!!");
}
