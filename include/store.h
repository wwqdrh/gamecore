#pragma once

#include <string>

namespace gamedb {
class FileStore {

private:
  std::string filename_ = "store.json";

public:
  FileStore(const std::string &filename) : filename_(filename){};

  void saveData(const std::string &data);
  std::string loadData();

private:
  std::string encrypt(const std::string &data);
  std::string decrypt(const std::string &data);
};
} // namespace libs