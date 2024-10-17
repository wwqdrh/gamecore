#include <gtest/gtest.h>

#include "dataclass.h"

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
  std::string get_name() const {
    return name;
  }
};

TEST(DataClassTest, TestFromMap) {
  Person p;
  std::map<std::string, std::any> data = {
      {"name", "Alice"}, {"age", 30}};

  p.fromMap(data);

  EXPECT_EQ(p.get_name(), "Alice");
  EXPECT_EQ(p.age, 30);
  EXPECT_EQ(p.height, 1.0);
}
