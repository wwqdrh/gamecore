#pragma once

#include <functional>
#include <string>

namespace gamedb {
class FileStore {
public:
  // 定义函数类型别名，使代码更清晰
  using LoadHandler = std::function<std::vector<uint8_t>()>;
  using SaveHandler = std::function<void(std::vector<uint8_t>)>;

private:
  std::string filename_ = "store.json";
  // 存储自定义处理函数
  LoadHandler customLoadHandler_ = nullptr;
  SaveHandler customSaveHandler_ = nullptr;

public:
  FileStore(const std::string &filename) : filename_(filename){};
  // 带自定义处理函数的构造函数
  FileStore(const LoadHandler &loadHandler, const SaveHandler &saveHandler)
      : customLoadHandler_(loadHandler), customSaveHandler_(saveHandler) {}

  FileStore(LoadHandler &&loadHandler, SaveHandler &&saveHandler)
      : customLoadHandler_(std::move(loadHandler)),
        customSaveHandler_(std::move(saveHandler)) {}
  ~FileStore() {}
  // 单独设置自定义处理函数的方法
  void setCustomLoadHandler(LoadHandler handler) {
    customLoadHandler_ = std::move(handler);
  }

  void setCustomSaveHandler(SaveHandler handler) {
    customSaveHandler_ = std::move(handler);
  }

  void saveData(const std::string &data);
  std::string loadData();

private:
  std::vector<uint8_t> encrypt(const std::string &data) const;
  std::string decrypt(const std::vector<uint8_t> &data) const;
};
} // namespace gamedb