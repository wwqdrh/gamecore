#include <atomic>
#include <barrier>
#include <gtest/gtest.h>
#include <iostream>
#include <memory>
#include <thread>
#include <variant>
#include <vector>

#include "argument.h"
#include "console.h"
#include "types.h"
using namespace gameconsole;

TEST(ConsoleTest, TestConsoleComplete) {
  auto console = Console(
      [](const std::string &val) -> void { std::cout << val << std::endl; });

  console.add_command(
      "command1",
      [](std::vector<Variant> args) {
        return Variant(std::string("command1"));
      },
      {std::make_shared<Argument>("name", ValueType::String,
                                  "a name' bool argument")},
      "test command1");
  console.add_command(
      "command1_void",
      [](std::vector<Variant> args) {
        return Variant(std::string("command1"));
      },
      {}, "test command1 no argument");

  ASSERT_EQ(console.autocomplete("command"), "command1");
  ASSERT_EQ(console.autocomplete("command1"), "command1_void");
  ASSERT_EQ(console.autocomplete("command1_"), "command1_void");
  ASSERT_EQ(console.autocomplete("command2"), "command2");
  ASSERT_EQ(console.get_commands("")->size(), 2);
  ASSERT_EQ(console.get_commands("command1_")->size(), 1);
}

TEST(ConsoleTest, TestConsoleExecute) {
  auto console = Console(
      [](const std::string &val) -> void { std::cout << val << std::endl; });

  int value = 0;

  console.add_command(
      "add",
      [&value](std::vector<Variant> args) {
        if (args.size() == 1) {
          if (std::holds_alternative<int>(args[0])) {
            value += std::get<int>(args[0]);
          }
        } else if (args.size() == 0) {
          value += 1;
        }
        return Variant(std::string("command1"));
      },
      {std::make_shared<Argument>("value", ValueType::Int,
                                  "a value' int argument")},
      "add operator");

  console.execute("add(1)");
  ASSERT_EQ(value, 1);
  console.execute("add(2)");
  ASSERT_EQ(value, 3);
  console.execute("add(3)");
  ASSERT_EQ(value, 6);
  console.execute("add()");
  ASSERT_EQ(value, 7);
}

TEST(ConsoleTest, TestConcurrentConsoleExecute) {
  auto console = Console([](const std::string &val) -> void {
    // std::cout << val << std::endl;
  });

  // 使用原子变量确保线程安全
  std::atomic<int> value{0};

  console.add_command(
      "add",
      [&value](std::vector<Variant> args) {
        if (args.size() == 1) {
          if (std::holds_alternative<int>(args[0])) {
            value.fetch_add(std::get<int>(args[0]));
          }
        } else if (args.size() == 0) {
          value.fetch_add(1);
        }
        return Variant(std::string("command1"));
      },
      {std::make_shared<Argument>("value", ValueType::Int,
                                  "a value' int argument")},
      "add operator");

  const int NUM_THREADS = 4;
  const int ITERATIONS_PER_THREAD = 2000;

  // 存储所有线程的vector
  std::vector<std::thread> threads;

  // 启动多个线程并发执行命令
  for (int i = 0; i < NUM_THREADS; i++) {
    threads.emplace_back([&console]() {
      for (int j = 0; j < ITERATIONS_PER_THREAD; j++) {
        console.execute("add(1)");
        console.execute("add()");
      }
    });
  }

  // JOIN所有线程
  for (auto &t : threads) {
    t.join();
  }

  // 验证最终结果
  // 每个线程执行ITERATIONS_PER_THREAD次add(1)和add()
  // 所以每个线程贡献了ITERATIONS_PER_THREAD * 2的增量
  // 总共有NUM_THREADS个线程
  const int EXPECTED_VALUE = NUM_THREADS * ITERATIONS_PER_THREAD * 2;
  ASSERT_EQ(value.load(), EXPECTED_VALUE);

  // 额外测试:确保在并发执行后单线程执行仍然正常
  console.execute("add(1)");
  ASSERT_EQ(value.load(), EXPECTED_VALUE + 1);
}

TEST(ConsoleTest, TestConsoleExecuteInclass) {
  class A {
  protected:
    std::shared_ptr<Console> console;

  public:
    static A build(std::function<void(const std::string &)> fn) {
      A c;
      c.console = std::make_shared<Console>(fn);
      return c;
    }

    void
    add_command(const std::string &name,
                const std::function<Variant(std::vector<Variant>)> &target) {
      if (console) {
        console->add_command(name, target, {}, "");
      }
    }

    void execute(const std::string& input) {
      if (console) {
        console->execute(input);
      }
    }
  };

  A con = A::build([](const std::string &val) -> void {});
  int value = 1;
  con.add_command("ping", [&value](std::vector<Variant> args) {
    value += 1;
    return Variant(std::string("pong"));
  });
  con.execute("ping()");
  ASSERT_EQ(value, 2);
  con.execute("ping()");
  ASSERT_EQ(value, 3);
}