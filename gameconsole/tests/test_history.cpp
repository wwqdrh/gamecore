#include <gtest/gtest.h>

#include "history.h"
using namespace gameconsole;

TEST(HistoryTest, TestHistoryLength) {
  auto history = History([](std::string s) { std::cout << s << std::endl; }, 5);
  history.push("history1");
  history.push("history2");
  history.push("history3");
  history.push("history4");
  history.push("history5");
  history.push("history6");

  ASSERT_EQ(history.size(), 5);
  ASSERT_EQ(std::get<std::string>(history.first().value()), "history2");
}
