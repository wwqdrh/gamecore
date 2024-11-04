#include <atomic>
#include <fstream>
#include <functional>
#include <map>
#include <memory>
#include <string>
#include <thread>
#include <vector>

#include "rapidjson/document.h"
#include <gtest/gtest.h>

#include "gjson.h" // 假设 GJson 类的声明在这个头文件中
#include "store.h"

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
  EXPECT_TRUE(json.query("").length() > 2);
  EXPECT_EQ(json.query("name"), "\"John Doe\"");
  EXPECT_EQ(json.query("age"), "30");
  EXPECT_EQ(json.queryT<int>("age"), 30);
  EXPECT_EQ(json.query("city"), "\"New York\"");
  EXPECT_EQ(json.query("is_student"), "false");
  EXPECT_EQ(json.query("grades;0"), "85");
  EXPECT_TRUE(json.query("grades").length() > 0);
  EXPECT_TRUE(json.query("address").length() > 0);
  EXPECT_EQ(json.query("address;street"), "\"123 Main St\"");
  EXPECT_TRUE(json.has("address;street"));
  EXPECT_FALSE(json.has("address;streeterr"));

  // update_from_function
  // !! 不能有多余逗号。否则不能解析
  GJson json2;
  json2.update_from_string(R"({
	"1": {
		"name": "任务1"
	}
})");
  // std::cout << json2.query("") << std::endl;
  EXPECT_TRUE(json2.query("").length() > 2);
  EXPECT_EQ(json2.query("1;name"), "\"任务1\"");
}

TEST(GJsonTest, ParseAndQueryWithFlag) {
  // 创建一个 GJson 对象
  GJson json(R"([{
        "name": "user1",
        "age": 30,
        "city": "New York"
    },
    {
        "name": "John Doe",
        "age": 31,
        "city": "New York"
    },
    {
        "name": "user2",
        "age": 30,
        "city": "New York"
    },
    {
        "name": "John Doe",
        "age": 32,
        "city": "New York"
    }
    ])");

  // 使用 query 方法测试各种数据类型的获取
  ASSERT_TRUE(json.query_value("#random(2)") == nullptr);
  ASSERT_TRUE(json.query_value_dynamic("#random(2)").Size() == 2);
  ASSERT_TRUE(json.query_value_dynamic("#all(age,=,30)").Size() == 2);
  ASSERT_TRUE(json.query("#all(age,=,30)").length() > 0);
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
    static Person fromJson(const rapidjson::Value &value) {
      Person person;
      if (value.IsNull() || !value.IsObject()) {
        return person;
      }

      if (value.HasMember("name")) {
        person.name = GJson::convert<std::string>(value["name"]);
      }

      if (value.HasMember("age")) {
        person.age = GJson::convert<int>(value["age"]);
      }

      if (value.HasMember("hobbies")) {
        person.hobbies =
            GJson::convert<std::vector<std::string>>(value["hobbies"]);
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
  Person person2 = GJson::convert<Person>(personValue);
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

TEST(GJsonTest, AutoFileSave) {
  std::string test_file = "test_filesave.json";

  GJson json;
  json.load_or_store(std::make_shared<FileStore>(test_file));
  json.updateT<int>("field1", "", 1);
  json.updateT<std::string>("field2", "", "name");

  // 检查test_filesave.json
  std::ifstream file(test_file);
  ASSERT_TRUE(file.is_open());
  file.close();

  // 从test_filesave.json中加载数据
  GJson json2;
  json2.load_or_store(std::make_shared<FileStore>(test_file));
  ASSERT_EQ(json2.queryT<int>("field1"), 1);
  ASSERT_EQ(json2.queryT<std::string>("field2"), "name");

  // 删除test_filesave.json文件
  std::remove(test_file.c_str());
}

TEST(GJsonTest, ConcurrentParseAndQuery) {
  const int NUM_THREADS = 10;
  const int ITERATIONS_PER_THREAD = 1000;
  std::atomic<int> success_count{0};

  // 创建一个共享的 GJson 对象
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

  // 定义测试函数
  auto test_function = [&](int thread_id) {
    for (int i = 0; i < ITERATIONS_PER_THREAD; ++i) {
      try {
        // 随机选择不同的查询操作
        switch (i % 7) {
        case 0:
          EXPECT_EQ(json.query("name"), "\"John Doe\"");
          break;
        case 1:
          EXPECT_EQ(json.query("age"), "30");
          EXPECT_EQ(json.queryT<int>("age"), 30);
          break;
        case 2:
          EXPECT_EQ(json.query("city"), "\"New York\"");
          break;
        case 3:
          EXPECT_EQ(json.query("is_student"), "false");
          break;
        case 4:
          EXPECT_EQ(json.query("grades;0"), "85");
          break;
        case 5:
          EXPECT_EQ(json.query("address;street"), "\"123 Main St\"");
          break;
        case 6:
          EXPECT_EQ(json.query("address;zip"), "\"10001\"");
          break;
        }
        success_count++;
      } catch (const std::exception &e) {
        ADD_FAILURE() << "Thread " << thread_id
                      << " failed with exception: " << e.what();
      }
    }
  };

  // 创建多个线程
  std::vector<std::thread> threads;
  for (int i = 0; i < NUM_THREADS; ++i) {
    threads.emplace_back(test_function, i);
  }

  // 等待所有线程完成
  for (auto &thread : threads) {
    thread.join();
  }

  // 验证所有操作都成功完成
  EXPECT_EQ(success_count.load(), NUM_THREADS * ITERATIONS_PER_THREAD);
}

TEST(GJsonTest, WatchBasicProperty) {
  // 测试是否能够监听属性的变化
  GJson json(R"({
    "name": "John Doe",
    "age": 30,
    "ext": {
      "address": "addressa"
    }
})");

  int address_change = 0;
  json.subscribe("ext;address",
                 [&address_change](const std::string &path,
                                   const rapidjson::Value *value) {
                   if (path == "ext;address" && value->IsString() &&
                       std::string(value->GetString()) == "addressb") {
                     address_change++;
                   }
                 });

  Document doc;
  rapidjson::Document::AllocatorType &allo = doc.GetAllocator();
  auto newAddress = GJson::toValue<std::string>("addressb", allo);
  // ~符号: 不管存不存在这个属性都会进行更新
  json.update("ext;address", "~", newAddress);
  ASSERT_EQ(address_change, 1);

  std::map<std::string, GJson::variant> newExt({{"address", "addressb"}});
  auto newExtVal =
      GJson::toValue<std::map<std::string, GJson::variant>>(newExt, allo);
  json.update("ext", "~", newExtVal);
  ASSERT_EQ(address_change, 2);
}