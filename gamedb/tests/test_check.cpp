#include "traits.h"
#include <gtest/gtest.h>

using namespace gamedb;
TEST(CheckTest, TestBasicParse) {
  ConditionParser p;
  variantDict args = {{"TLT", std::vector<std::string>{"1024"}}};
  ASSERT_TRUE(p.checkCondition(args, "TLT?[1004,1024,1025,1113]"));
  variantDict args2 = {{"TLT", std::vector<std::string>{"1026"}}};
  ASSERT_FALSE(p.checkCondition(args2, "TLT?[1004,1024,1025,1113]"));
  variantDict args3 = {{"TLT", std::vector<std::string>{"10452"}},
                       {"EVT", std::vector<std::string>{"10460"}},
                       {"VAL", "123456"}};
  ASSERT_TRUE(p.checkCondition(args3, "(TLT?[10452])&(EVT?[10460])"));
  ASSERT_FALSE(p.checkCondition(args3, "(TLT?[10452])&(EVT?[10461])"));
  ASSERT_TRUE(p.checkCondition(args3, "VAL=123456"));
}
