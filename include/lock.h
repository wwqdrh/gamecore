#pragma once

#include <mutex>
#include <shared_mutex>
#include <thread>
#include <unordered_map>

namespace gamedb {
// 合并了原来的 writer_tracking_mutex_ 和 reader_tracking_mutex_ 为单个
// state_mutex_，确保锁的获取顺序一致，避免死锁。 在 lock_shared() 和 lock()
// 方法中，采用了"二段锁"策略： 首先获取状态锁检查重入
// 如果需要获取新锁，则先释放状态锁
// 获取读/写锁
// 重新获取状态锁更新状态
// 使用 std::unique_lock 代替部分
// std::lock_guard，以支持更灵活的锁操作（解锁后重新加锁）。
class ReentrantRWLock {
private:
  // 底层的共享互斥量
  std::shared_mutex mutex_;

  // 用于保护内部状态的互斥锁
  std::mutex state_mutex_;

  // 写锁状态
  std::thread::id current_writer_;
  size_t writer_count_ = 0;

  // 读锁状态
  std::unordered_map<std::thread::id, size_t> reader_counts_;

public:
  ReentrantRWLock() = default;

  // 禁止拷贝和移动
  ReentrantRWLock(const ReentrantRWLock &) = delete;
  ReentrantRWLock &operator=(const ReentrantRWLock &) = delete;
  ReentrantRWLock(ReentrantRWLock &&) = delete;
  ReentrantRWLock &operator=(ReentrantRWLock &&) = delete;

  // 获取读锁
  void lock_shared() {
    std::thread::id this_id = std::this_thread::get_id();

    // 首先获取状态锁
    std::unique_lock<std::mutex> state_lock(state_mutex_);

    // 检查当前线程是否已经持有写锁
    if (current_writer_ == this_id) {
      // 如果当前线程持有写锁，直接返回(写锁包含读权限)
      return;
    }

    // 检查是否需要重入读锁
    auto it = reader_counts_.find(this_id);
    if (it != reader_counts_.end() && it->second > 0) {
      // 重入读锁
      ++(it->second);
      return;
    }

    // 在获取共享锁之前释放状态锁，避免死锁
    state_lock.unlock();

    // 获取共享锁
    mutex_.lock_shared();

    // 重新获取状态锁并更新读者计数
    state_lock.lock();
    reader_counts_[this_id] = 1;
  }

  // 释放读锁
  void unlock_shared() {
    std::thread::id this_id = std::this_thread::get_id();

    std::lock_guard<std::mutex> state_lock(state_mutex_);

    // 检查是否持有写锁
    if (current_writer_ == this_id) {
      // 持有写锁时无需释放读锁
      return;
    }

    auto it = reader_counts_.find(this_id);
    if (it != reader_counts_.end() && it->second > 0) {
      --(it->second);
      if (it->second == 0) {
        // 完全释放读锁
        reader_counts_.erase(it);
        mutex_.unlock_shared();
      }
    }
  }

  // 获取写锁
  void lock() {
    std::thread::id this_id = std::this_thread::get_id();

    // 首先获取状态锁
    std::unique_lock<std::mutex> state_lock(state_mutex_);

    // 检查写锁重入
    if (current_writer_ == this_id) {
      ++writer_count_;
      return;
    }

    // 在获取互斥锁之前释放状态锁，避免死锁
    state_lock.unlock();

    // 获取互斥锁
    mutex_.lock();

    // 重新获取状态锁并更新写者状态
    state_lock.lock();
    current_writer_ = this_id;
    writer_count_ = 1;
  }

  // 释放写锁
  void unlock() {
    std::thread::id this_id = std::this_thread::get_id();

    std::lock_guard<std::mutex> state_lock(state_mutex_);

    if (current_writer_ != this_id) {
      return;
    }

    --writer_count_;
    if (writer_count_ == 0) {
      current_writer_ = std::thread::id();
      mutex_.unlock();
    }
  }

  // RAII风格的读锁guard
  [[nodiscard]] auto shared_lock() { return SharedLockGuard(*this); }

  // RAII风格的写锁guard
  [[nodiscard]] auto unique_lock() { return UniqueLockGuard(*this); }

private:
  // 读锁的RAII包装器
  class SharedLockGuard {
  public:
    explicit SharedLockGuard(ReentrantRWLock &lock) : lock_(lock) {
      lock_.lock_shared();
    }

    ~SharedLockGuard() { lock_.unlock_shared(); }

    SharedLockGuard(const SharedLockGuard &) = delete;
    SharedLockGuard &operator=(const SharedLockGuard &) = delete;

  private:
    ReentrantRWLock &lock_;
  };

  // 写锁的RAII包装器
  class UniqueLockGuard {
  public:
    explicit UniqueLockGuard(ReentrantRWLock &lock) : lock_(lock) {
      lock_.lock();
    }

    ~UniqueLockGuard() { lock_.unlock(); }

    UniqueLockGuard(const UniqueLockGuard &) = delete;
    UniqueLockGuard &operator=(const UniqueLockGuard &) = delete;

  private:
    ReentrantRWLock &lock_;
  };
};
} // namespace gamedb