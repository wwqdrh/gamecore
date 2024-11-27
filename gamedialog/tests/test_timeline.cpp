#include <gtest/gtest.h>

#include "timeline.h"

using namespace gamedialog;

TEST(TimelineTest, testparse) {
  Timeline parser(R"(
[stage1]
(John)
Hello there!
-Yes@next_scene
-No@end_scene

[next_scene]
(Mary)
answer yes!!

[end_scene]
answer no!!
)");

  auto stages = parser.all_stages();
  ASSERT_EQ(3, stages.size());

  // 执行一次调用
  auto word = parser.next();
  ASSERT_TRUE(word != nullptr);
  ASSERT_EQ("John", word->get_name());
  ASSERT_EQ("Hello there!", word->get_text());
  ASSERT_EQ(2, word->get_responses().size());

  // 手动跳转
  parser.goto_stage("next_scene");
  auto word2 = parser.next();
  ASSERT_EQ("Mary", word2->get_name());
  ASSERT_EQ("answer yes!!", word2->get_text());

  // 回到开头
  parser.goto_begin();
  ASSERT_EQ(parser.current_stage(), "stage1");
}

TEST(TimelineTest, TestStagePreCheck) {
  Timeline parser(R"(
[stage1@func1]
(John)
Hello there!
:skip:2

[next_scene@func2]
(Mary)
answer yes!!
:end
)");

  std::string cur_state = "func2";
  parser.set_precheck([&cur_state](const std::string &expr) -> bool {
    return cur_state == expr;
  });

  ASSERT_EQ(parser.next()->get_name(), "Mary");
}