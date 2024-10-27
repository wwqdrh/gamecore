#include <gtest/gtest.h>
#include <random>
#include <thread>

#include "lock.h"

using namespace gamedb;

TEST(LockTest, BasicFunc) {
  ReentrantRWLock lock;
  std::atomic<int> shared_data = 0;

  // 测试1：基本的读写操作
  {
    auto writer = lock.unique_lock();
    shared_data = 42;
  }

  {
    auto reader = lock.shared_lock();
    ASSERT_EQ(shared_data, 42);
  }

  // 测试2：多重读锁
  {
    auto reader1 = lock.shared_lock();
    auto reader2 = lock.shared_lock();
    ASSERT_EQ(shared_data, 42);
  }

  // 测试3：写锁重入
  {
    auto writer1 = lock.unique_lock();
    shared_data = 100;
    {
      auto writer2 = lock.unique_lock();
      shared_data = 200;
    }
    ASSERT_EQ(shared_data, 200);
  }

  // 测试4：持有写锁时获取读锁
  {
    auto writer = lock.unique_lock();
    shared_data = 300;
    {
      auto reader = lock.shared_lock();
      ASSERT_EQ(shared_data, 300);
    }
  }
}

TEST(LockTest, TestConcurrentSafe) {
  ReentrantRWLock lock;
  int shared_data = 0;
  std::atomic<bool> stop = false;
  int write_count = 0;

  // 创建多个读写线程
  std::vector<std::thread> threads;

  // 读线程
  auto reader = [&](int id) {
    while (!stop) {
      auto guard = lock.shared_lock();
      int value = shared_data;
      (void)value; // 避免未使用警告
                   //   ++read_count;
      std::this_thread::yield();
    }
  };

  // 写线程
  auto writer = [&](int id) {
    while (!stop) {
      {
        // 避免其他线程无法拿到锁
        auto guard = lock.unique_lock();
        // lock.lock();
        shared_data = id;
        ++write_count;
        // lock.unlock();
      }
      std::this_thread::yield();
      //   std::this_thread::sleep_for(
      //       std::chrono::microseconds(std::random_device{}() % 200));
    }
  };

  // 启动线程
  const int reader_count = 4;
  const int writer_count = 2;

  for (int i = 0; i < reader_count; ++i) {
    threads.emplace_back(reader, i);
  }
  for (int i = 0; i < writer_count; ++i) {
    threads.emplace_back(writer, i);
  }

  // 运行一段时间
  std::this_thread::sleep_for(std::chrono::seconds(2));
  stop = true;

  // 等待所有线程结束
  for (auto &t : threads) {
    t.join();
  }

  ASSERT_TRUE(write_count > 0);
}
