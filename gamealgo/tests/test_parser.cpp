
#include <gtest/gtest.h>

#include "parser/typewriter.h"

using namespace gamealgo;

TEST(TestParser, typewriter) {
  TextParser parser;
  std::string input =
      "[color=#ff0000]你[/color][shake]e[/shake][size=20]llo[/size][你好]";
  //   std::string input =
  //       "你好";
  auto tokens = parser.parse_text(input);

  ASSERT_EQ(tokens.size(), 7);
  ASSERT_TRUE(std::get<TokenType>(tokens[0].at("type")) == TokenType::COLOR &&
              std::get<std::string>(tokens[0].at("color")) == "#ff0000" &&
              std::get<std::string>(tokens[0].at("content")) ==
                  std::string("你"));
  ASSERT_TRUE(std::get<TokenType>(tokens[1].at("type")) == TokenType::SHAKE &&
              std::get<std::string>(tokens[1].at("content")) ==
                  std::string("e"));
  ASSERT_TRUE(std::get<TokenType>(tokens[2].at("type")) == TokenType::SIZE &&
              std::get<int>(tokens[2].at("size")) == 20 &&
              std::get<std::string>(tokens[2].at("content")) ==
                  std::string("l"));
  ASSERT_TRUE(std::get<TokenType>(tokens[3].at("type")) == TokenType::SIZE &&
              std::get<int>(tokens[3].at("pid")) == 2 &&
              std::get<std::string>(tokens[3].at("content")) ==
                  std::string("l"));
  ASSERT_TRUE(std::get<TokenType>(tokens[4].at("type")) == TokenType::SIZE &&
              std::get<std::string>(tokens[4].at("content")) ==
                  std::string("o"));
  ASSERT_TRUE(std::get<TokenType>(tokens[5].at("type")) == TokenType::TEXT && std::get<int>(tokens[5].at("pid")) == 3);
  ASSERT_TRUE(std::get<std::string>(tokens[5].at("content")) ==
              std::string("你"));
}