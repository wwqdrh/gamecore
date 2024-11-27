#include <gtest/gtest.h>

#include "types.h"

using namespace gameconsole;

TEST(TypesTest, TestBasicNormal) {
  BoolType b;

  ASSERT_EQ(std::get<bool>(b.normalize("true")), true);
  ASSERT_EQ(std::get<bool>(b.normalize("TRUE")), true);
  ASSERT_EQ(std::get<bool>(b.normalize("1")), true);
  ASSERT_EQ(std::get<bool>(b.normalize("false")), false);
  ASSERT_EQ(std::get<bool>(b.normalize("0")), false);
  ASSERT_EQ(std::get<bool>(b.normalize("invalid")), false);

  StringType s;
  ASSERT_EQ(std::get<std::string>(s.normalize("test")), "test");
  ASSERT_EQ(std::get<std::string>(s.normalize(123)), "123");
  ASSERT_EQ(std::get<std::string>(s.normalize(true)), "true");
}

TEST(TypesTest, TestRegexNormal) {
  RegexType b;
  b.initialize("Number", R"(^\d+$)");

  ASSERT_EQ(b.check("123"), CheckResult::Ok);
  ASSERT_EQ(b.check("abc"), CheckResult::Failed);
  ASSERT_EQ(b.check("12.34"), CheckResult::Failed);
}

TEST(TypesTest, TestFilter) {
  FilterType f;
  std::vector<Variant> allowed_values = {"allowed1", "allowed2"};

  // allow
  f.initialize(allowed_values, FilterType::Mode::Allow);
  ASSERT_EQ(f.check("allowed1"), CheckResult::Ok);
  ASSERT_EQ(f.check("not_allowed"), CheckResult::Canceled);

  // disallow
  f.initialize(allowed_values, FilterType::Mode::Deny);
  ASSERT_EQ(f.check("allowed1"), CheckResult::Canceled);
  ASSERT_EQ(f.check("not_allowed"), CheckResult::Ok);
}

TEST(TypesTest, TestFactory) {
  auto any_type = TypeFactory::build(ValueType::Any);
  ASSERT_EQ(any_type->toString(), "Any");

  auto bool_type = TypeFactory::build(ValueType::Bool);
  ASSERT_EQ(bool_type->toString(), "Bool");

  auto string_type = TypeFactory::build(ValueType::String);
  ASSERT_EQ(string_type->toString(), "String");
}