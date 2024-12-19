#pragma once

#include <functional>
#include <iostream>
#include <memory>
#include <ostream>
#include <sstream>
#include <string>
#include <iomanip>

namespace gamedb {

enum class LogLevel { DEBUG, INFO, WARNING, ERROR, FATAL };

class GLogger {
public:
  using LogCallback = std::function<void(
      LogLevel level, const std::string &module, const std::string &message)>;

private:
  static inline std::shared_ptr<GLogger> instance_ = nullptr;
  LogCallback log_callback_;
  bool enable_debug_ = false;

public:
  static std::shared_ptr<GLogger> getInstance() {
    if (!instance_) {
      instance_ = std::shared_ptr<GLogger>(new GLogger());
    }
    return instance_;
  }

  // 设置日志回调函数
  void setLogCallback(LogCallback callback) { log_callback_ = callback; }

  // 启用/禁用调试日志
  void enableDebug(bool enable) { enable_debug_ = enable; }

  // 日志记录函数
  void debug(const std::string &module, const std::string &message) {
    if (enable_debug_) {
      log(LogLevel::DEBUG, module, message);
    }
  }

  void info(const std::string &module, const std::string &message) {
    log(LogLevel::INFO, module, message);
  }

  void warning(const std::string &module, const std::string &message) {
    log(LogLevel::WARNING, module, message);
  }

  void error(const std::string &module, const std::string &message) {
    log(LogLevel::ERROR, module, message);
  }

  void fatal(const std::string &module, const std::string &message) {
    log(LogLevel::FATAL, module, message);
  }

private:
  GLogger() = default;

  void log(LogLevel level, const std::string &module,
           const std::string &message) {
    if (log_callback_) {
      log_callback_(level, module, message);
    } else {
      defaultLogHandler(level, module, message);
    }
  }

  void defaultLogHandler(LogLevel level, const std::string &module,
                         const std::string &message) {
    // 获取当前时间
    auto now = std::chrono::system_clock::now();
    auto time = std::chrono::system_clock::to_time_t(now);
    auto ms = std::chrono::duration_cast<std::chrono::milliseconds>(
                  now.time_since_epoch()) %
              1000;

    std::stringstream ss;
    ss << std::put_time(std::localtime(&time), "%Y-%m-%d %H:%M:%S");
    ss << '.' << std::setfill('0') << std::setw(3) << ms.count();

    const char *level_str = "";
    switch (level) {
    case LogLevel::DEBUG:
      level_str = "DEBUG";
      break;
    case LogLevel::INFO:
      level_str = "INFO";
      break;
    case LogLevel::WARNING:
      level_str = "WARNING";
      break;
    case LogLevel::ERROR:
      level_str = "ERROR";
      break;
    case LogLevel::FATAL:
      level_str = "FATAL";
      break;
    }

    std::cerr << "[" << ss.str() << "] [" << level_str << "] [" << module
              << "] " << message << std::endl;
  }
};

// 便捷宏定义
#define GLOG_DEBUG(module, message)                                            \
  gamedb::GLogger::getInstance()->debug(module, message)
#define GLOG_INFO(module, message)                                             \
  gamedb::GLogger::getInstance()->info(module, message)
#define GLOG_WARNING(module, message)                                          \
  gamedb::GLogger::getInstance()->warning(module, message)
#define GLOG_ERROR(module, message)                                            \
  gamedb::GLogger::getInstance()->error(module, message)
#define GLOG_FATAL(module, message)                                            \
  gamedb::GLogger::getInstance()->fatal(module, message)
} // namespace gamedb