#include <gtest/gtest.h>

#include "timeline.h"
#include "flow.h"

using namespace gamedialog;

TEST(FlowTest, TestFlowCreate) {
  auto flow = ControlFlowFactory::createFromString(":start");
  ASSERT_EQ(flow->getName(), "start");

  auto flow2 = ControlFlowFactory::createFromString(":skip:2");
  ASSERT_EQ(flow2->getName(), "skip");
  std::shared_ptr<SkipFlow> skipPtr1 =
      std::dynamic_pointer_cast<SkipFlow>(flow2);
  ASSERT_TRUE(skipPtr1 != nullptr);
  ASSERT_EQ(skipPtr1->getSkipCount(), 2);

  auto flow3 = ControlFlowFactory::createFromString(":end");
  ASSERT_EQ(flow3->getName(), "end");

  auto flow4 = ControlFlowFactory::createFromString(":goto:stage1");
  ASSERT_EQ(flow4->getName(), "goto");
  std::shared_ptr<GotoFlow> gotoPtr1 =
      std::dynamic_pointer_cast<GotoFlow>(flow4);
  ASSERT_TRUE(gotoPtr1 != nullptr);
  ASSERT_EQ(gotoPtr1->getTargetName(), "stage1");
}
