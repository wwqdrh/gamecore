#include <gtest/gtest.h>
#include <iostream>
#include <vector>

#include "question/manager.h"

using namespace gamedb;

// 添加任务、设置任务状态
// 设置任务目标
TEST(QuestionTest, TestBasic) {
  QuesManager ques;
  std::vector<int> ava_ids;
  std::vector<int> active_ids;

  ASSERT_TRUE(ques.addTask(1));
  ava_ids = ques.get_available_task();
  ASSERT_EQ(ava_ids.size(), 1);
  ASSERT_FALSE(ques.addTask(1));  // 不能重复添加
  ASSERT_TRUE(ques.startTask(1)); // 开始任务
  ava_ids = ques.get_available_task();
  ASSERT_EQ(ava_ids.size(), 0);
  active_ids = ques.get_active_task();
  ASSERT_EQ(active_ids.size(), 1);
  ASSERT_FALSE(ques.startTask(2)); // 不存在的任务不能开始
  ASSERT_TRUE(ques.completeTask(
      1)); // 完成任务, 会判断任务是否完成，对于没有条件的任务默认是完成的
  ASSERT_FALSE(ques.completeTask(2)); //

  // 测试设置任务目标
  ques.addTask(2);
  ASSERT_TRUE(ques.addTaskTarget(2, "5个苹果", 5));
  ASSERT_TRUE(ques.addTaskTarget(2, "10个香蕉", 10));
  ASSERT_TRUE(ques.startTask(2));
  ques.updateTaskTarget(2, 0, 5);
  ASSERT_FALSE(ques.completeTask(2)); // 还有10个香蕉没有收集
  ques.updateTaskTarget(2, 1, 5);
  ASSERT_FALSE(ques.completeTask(2));
  ques.updateTaskTarget(2, 1, 5);
  ASSERT_TRUE(ques.completeTask(2));
}

TEST(QuestionTest, TestLoadFile) {
  std::string test_file = "test_question_loadfile.json";

  auto json = std::make_shared<GJson>(std::make_shared<FileStore>(test_file));

  QuesManager ques;
  ques.set_store(json);

  std::vector<int> ava_ids;
  std::vector<int> active_ids;
  std::cout << json->query("") << std::endl;
  ASSERT_TRUE(ques.addTask(1));
  ava_ids = ques.get_available_task();
  ASSERT_EQ(ava_ids.size(), 1);
  ASSERT_FALSE(ques.addTask(1));  // 不能重复添加
  ASSERT_TRUE(ques.startTask(1)); // 开始任务
  ava_ids = ques.get_available_task();
  ASSERT_EQ(ava_ids.size(), 0);
  active_ids = ques.get_active_task();
  ASSERT_EQ(active_ids.size(), 1);
  ASSERT_FALSE(ques.startTask(2));    // 不存在的任务不能开始
  ASSERT_TRUE(ques.completeTask(1));  // 完成任务
  ASSERT_FALSE(ques.completeTask(2)); //

  // 删除test_filesave.json文件
  std::remove(test_file.c_str());
}
