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

TEST(StageTest, TestSceneVariables) {
  gamedialog::DiaStage stage(R"(
[stage1]
```
name=John Doe
age=30
```
Hello there!
@set:age=31
)");

  ASSERT_EQ(stage.get_stage_name(), "stage1");
  ASSERT_EQ(stage.get_line_size(), 1);
  ASSERT_EQ(stage.get_variable("name"), "John Doe");
  ASSERT_EQ(stage.get_variable("age"), "30");
  ASSERT_EQ(stage.next()->get_text(), "Hello there!");
  ASSERT_EQ(stage.get_variable("age"), "31");
}

TEST(StageTest, TestMultipleConditions) {
  gamedialog::DiaStage stage(R"(
[stage1]
```
status=happy
points=100
```
(John)
Hello
@if:status=happy&points=100:high_score:low_score

@label:high_score
High score!
@goto:end

@label:low_score
Low score!
@goto:end

@label:end
Done.
)");

  auto word = stage.next();
  ASSERT_EQ(word->get_text(), "Hello");
  
  // Should go to high_score because both conditions are met
  word = stage.next();
  ASSERT_EQ(word->get_text(), "High score!");
  
  word = stage.next();
  ASSERT_EQ(word->get_text(), "Done.");
}

TEST(StageTest, TestEntryConditions) {
    // Set up a global variable first
    SceneManager::instance().set_variable("level", "5");
    
    gamedialog::DiaStage stage(R"(
[stage1]
```
points=100
?global.level>3&points>50
```
(John)
Hello there!
)");

    ASSERT_TRUE(stage.check_entry_conditions());
    
    // Test with failing condition
    SceneManager::instance().set_variable("level", "2");
    ASSERT_FALSE(stage.check_entry_conditions());
    
    // Test with multiple operators
    gamedialog::DiaStage stage2(R"(
[stage2]
```
score=75
?score>=75&global.level<=5
```
(Mary)
Hi!
)");

    ASSERT_TRUE(stage2.check_entry_conditions());
}
