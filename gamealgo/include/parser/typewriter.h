#pragma once

#include <iostream>
#include <optional>
#include <regex>
#include <string>
#include <unordered_map>
#include <vector>

namespace gamealgo {
enum class TokenType { TEXT, COLOR, SIZE, SHAKE, WAVE, PLAIN_TEXT };

class TextParser {
public:
  using Token = std::unordered_map<
      std::string,
      std::variant<std::monostate, int, std::string, bool, TokenType>>;

public:
  std::vector<Token> parse_text(const std::string &text) {
    std::vector<Token> tokens;
    size_t current_pos = 0;
    Token active_styles;
    TokenType current_type = TokenType::TEXT;

    int current_pid = 0;
    while (current_pos < text.length()) {
      // 检查是否遇到标签开始符号
      if (text[current_pos] == '[') {
        size_t tag_end = text.find(']', current_pos);
        if (tag_end != std::string::npos) {
          std::string tag = text.substr(current_pos, tag_end - current_pos + 1);
          if (is_closing_tag(tag)) {
            // 如果是闭合标签，重置对应的样式和类型
            reset_style_by_tag(active_styles, tag);
            current_type = TokenType::TEXT;
            current_pos = tag_end + 1;
            continue;
          }

          auto next_token = parse_tag(text, current_pos);
          if (next_token.has_value()) {
            // 更新样式和当前类型
            update_active_styles(active_styles, *next_token);
            current_type = next_token->type;
            // 处理标签内的文本内容
            for (std::string c : toRune(next_token->content)) {
              tokens.push_back(
                  create_token(current_pid, current_type, c, active_styles));
            }
            current_pid++;
            current_pos = next_token->end_pos;
            continue;
          }
        }
      }

      std::string rune = getUtf8Char(text, current_pos);
      if (rune != "[" && rune != "]") {
        // 处理普通文本
        tokens.push_back(
            create_token(current_pid, TokenType::TEXT, rune, active_styles));
      }
      current_pos += rune.size();
    }

    return tokens;
  }

private:
  struct NextToken {
    size_t start_pos;
    size_t end_pos;
    int pid;
    TokenType type;
    std::string content;
    std::string param; // 用于存储颜色值或尺寸等参数
  };

  bool is_closing_tag(const std::string &tag) { return tag.find("[/") == 0; }

  void reset_style_by_tag(Token &active_styles, const std::string &tag) {
    if (tag == "[/color]") {
      active_styles["color"] = std::monostate{};
    } else if (tag == "[/size]") {
      active_styles["size"] = std::monostate{};
    } else if (tag == "[/shake]") {
      active_styles["shake"] = false;
    } else if (tag == "[/wave]") {
      active_styles["wave"] = false;
    }
  }

  std::vector<std::string> toRune(const std::string &text) {
    std::vector<std::string> runes;
    size_t pos = 0;
    while (pos < text.length()) {
      std::string rune = getUtf8Char(text, pos);
      runes.push_back(rune);
      pos += rune.length();
    }
    return runes;
  }

  std::string getUtf8Char(const std::string &str, size_t pos) {
    if (pos >= str.length())
      return "";
    size_t len = 1;
    auto c = static_cast<unsigned char>(str[pos]);
    if ((c & 0x80) == 0)
      len = 1;
    if ((c & 0xE0) == 0xC0)
      len = 2;
    if ((c & 0xF0) == 0xE0)
      len = 3;
    if ((c & 0xF8) == 0xF0)
      len = 4;
    if (pos + len > str.length())
      return "";
    return str.substr(pos, len);
  }

  std::optional<NextToken> parse_tag(const std::string &text,
                                     size_t start_pos) {
    static const std::vector<std::pair<std::regex, TokenType>> patterns = {
        {std::regex(R"(\[color=(#[0-9a-fA-F]{6})\](.*?)\[\/color\])"),
         TokenType::COLOR},
        {std::regex(R"(\[size=(\d+)\](.*?)\[\/size\])"), TokenType::SIZE},
        {std::regex(R"(\[shake\](.*?)\[\/shake\])"), TokenType::SHAKE},
        {std::regex(R"(\[wave\](.*?)\[\/wave\])"), TokenType::WAVE}};

    int patternid = 0;

    for (const auto &[pattern, type] : patterns) {
      patternid++;
      std::smatch matches;
      if (std::regex_search(text.begin() + start_pos, text.end(), matches,
                            pattern)) {
        if (matches.position(0) + start_pos == start_pos) {
          NextToken token;
          token.start_pos = start_pos;
          token.end_pos = start_pos + matches.length(0);
          token.pid = patternid;
          token.type = type;

          // 对于不同类型的标签，提取相应的内容和参数
          if (type == TokenType::COLOR) {
            token.param = matches[1].str();   // 颜色值
            token.content = matches[2].str(); // 文本内容
          } else if (type == TokenType::SIZE) {
            token.param = matches[1].str();   // 尺寸值
            token.content = matches[2].str(); // 文本内容
          } else {
            token.content = matches[1].str(); // 文本内容
          }

          return token;
        }
      }
    }

    return std::nullopt;
  }

  Token create_token(int pid, TokenType type, const std::string &content,
                     const Token &active_styles) {
    Token token;
    token["pid"] = pid;
    token["type"] = type;
    token["content"] = content;
    for (const auto &[key, value] : active_styles) {
      token[key] = value;
    }
    return token;
  }

  void update_active_styles(Token &active_styles, const NextToken &next_token) {
    switch (next_token.type) {
    case TokenType::COLOR:
      active_styles["color"] = next_token.param; // 使用param而不是content
      break;
    case TokenType::SIZE:
      active_styles["size"] =
          std::stoi(next_token.param); // 使用param而不是content
      break;
    case TokenType::SHAKE:
      active_styles["shake"] = true;
      break;
    case TokenType::WAVE:
      active_styles["wave"] = true;
      break;
    default:
      break;
    }
  }
};
} // namespace gamealgo