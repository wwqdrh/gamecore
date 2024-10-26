#include <gtest/gtest.h>

#include "gjson.h" // 假设 GJson 类的声明在这个头文件中

using namespace gamedb;

TEST(GJsonTest, ParseAndQuery) {
  // 创建一个 GJson 对象
  GJson json(R"({
        "name": "John Doe",
        "age": 30,
        "city": "New York",
        "is_student": false,
        "grades": [85, 90, 78],
        "address": {
            "street": "123 Main St",
            "zip": "10001"
        }
    })");

  // 使用 query 方法测试各种数据类型的获取
  EXPECT_EQ(json.query("name"), "\"John Doe\"");
  EXPECT_EQ(json.query("age"), "30");
  EXPECT_EQ(json.queryv<int>("age"), 30);
  EXPECT_EQ(json.query("city"), "\"New York\"");
  EXPECT_EQ(json.query("is_student"), "false");
  EXPECT_EQ(json.query("grades;0"), "85");
  EXPECT_EQ(json.query("address;street"), "\"123 Main St\"");
}

TEST(GJsonTest, TypeConvert) {
  rapidjson::Document doc;
  auto &allocator = doc.GetAllocator();

  // 使用示例的自定义类
  class Person {
  public:
    std::string name;
    int age;
    std::vector<std::string> hobbies;

    Person() = default;
    Person(const std::string &name, int age,
           const std::vector<std::string> &hobbies)
        : name(name), age(age), hobbies(hobbies) {}

    rapidjson::Value
    toJson(rapidjson::Document::AllocatorType &allocator) const {
      rapidjson::Value obj(rapidjson::kObjectType);

      obj.AddMember("name", GJson::toValue(name, allocator), allocator);

      obj.AddMember("age", GJson::toValue(age, allocator), allocator);

      obj.AddMember("hobbies", GJson::toValue(hobbies, allocator), allocator);

      return obj;
    }

    // 反序列化静态方法
    static Person fromJson(rapidjson::Value *value) {
      Person person;
      if (!value || !value->IsObject()) {
        return person;
      }

      if (value->HasMember("name")) {
        person.name = GJson::convert<std::string>(&(*value)["name"]);
      }

      if (value->HasMember("age")) {
        person.age = GJson::convert<int>(&(*value)["age"]);
      }

      if (value->HasMember("hobbies")) {
        person.hobbies =
            GJson::convert<std::vector<std::string>>(&(*value)["hobbies"]);
      }

      return person;
    }
  };

  // 基本类型
  auto intValue = GJson::toValue(42, allocator);
  ASSERT_EQ(intValue.GetInt(), 42);
  auto strValue = GJson::toValue(std::string("hello"), allocator);
  ASSERT_TRUE(strValue.IsString() &&
              (strcmp(strValue.GetString(), "hello") == 0));
  auto doubleValue = GJson::toValue(3.14, allocator);
  ASSERT_EQ(doubleValue.GetDouble(), 3.14);

  // 容器类型
  std::vector<int> vec = {1, 2, 3};
  auto vecValue = GJson::toValue(vec, allocator);
  ASSERT_TRUE(vecValue.IsArray() && vecValue.Size() == 3);

  std::vector<std::string> vecStr = {"1", "2", "3"};
  auto vecStrValue = GJson::toValue(vecStr, allocator);
  ASSERT_TRUE(vecStrValue.IsArray() && vecStrValue.Size() == 3);

  std::map<std::string, int> map = {{"one", 1}, {"two", 2}};
  auto mapValue = GJson::toValue(map, allocator);
  ASSERT_TRUE(mapValue.IsObject() && mapValue.MemberCount() == 2);

  // 自定义类型
  Person person{"John", 30, {"reading", "coding"}};
  auto personValue = GJson::toValue(person, allocator);
  Person person2 = GJson::convert<Person>(&personValue);
  ASSERT_EQ(person2.name, "John");
  ASSERT_EQ(person2.age, 30);
  ASSERT_EQ(person2.hobbies.size(), 2);
  // Person person2(personValue);

  // 复杂嵌套类型
  std::vector<std::map<std::string, int>> complex = {{{"a", 1}, {"b", 2}},
                                                     {{"c", 3}, {"d", 4}}};
  auto complexValue = GJson::toValue(complex, allocator);
  ASSERT_TRUE(complexValue.IsArray() && complexValue.Size() == 2);
}