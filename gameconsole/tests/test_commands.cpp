#include <gtest/gtest.h>

#include "commands.h"
using namespace gameconsole;

TEST(CommandsTest, TestCommand) {
  auto commands = Commands(VariantMap(
      {{"command1", 1}, {"command1_1", 1}, {"command2", 2}, {"command3", 3}}));
  ASSERT_EQ(commands.size(), 4);
  ASSERT_EQ(commands.find("command1")->size(), 2);
}
