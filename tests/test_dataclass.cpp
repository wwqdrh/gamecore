#include <gtest/gtest.h>
#include <iostream>

#include "dataclass.h"
using namespace gamedb;
class Person : public DataClass<Person> {
private:
  std::string name = "default";

public:
  int age = 0;
  double height = 1.0;

public:
  Person() {
    addMember("name", &Person::name);
    addMember("age", &Person::age);
    addMember("height", &Person::height);
  }
  explicit Person(const std::string &data) : Person() {
    // GENERATE_ADD_MEMBER(Person, name, age, height);
    fromJson(data);
  }
  std::string get_name() const { return name; }
};

TEST(DataClassTest, TestFromMap) {
  Person p;
  std::map<std::string, std::any> data = {{"name", "Alice"}, {"age", 30}};

  p.fromMap(data);

  EXPECT_EQ(p.get_name(), "Alice");
  EXPECT_EQ(p.age, 30);
  EXPECT_EQ(p.height, 1.0);
}

TEST(DataClassTest, TestFromJson) {
  Person p(R"({
        "name": "Alice",
        "age": 30
    })");

  EXPECT_EQ(p.toJson(), R"({"age":30,"height":1.0,"name":"Alice"})");
  EXPECT_EQ(p.get_name(), "Alice");
  EXPECT_EQ(p.age, 30);
  EXPECT_EQ(p.height, 1.0);
}

TEST(DataClassTest, TestFromJsonArr) {
  std::vector<Person> res = Person::fromJsonArr(R"([{
        "name": "Alice",
        "age": 30
    },{
        "name": "Alice2",
        "age": 31
    }])");

  EXPECT_EQ(res.size(), 2);
  EXPECT_EQ(res[0].get_name(), "Alice");
  EXPECT_EQ(res[1].get_name(), "Alice2");
}
