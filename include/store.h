#pragma once

#include <string>

namespace gamedb {
class FileManager {

private:
  std::string filename_;

public:
  FileManager(const std::string &filename) : filename_(filename){};

  void saveData(const std::string &data);
  std::string loadData();

private:
  std::string encrypt(const std::string &data);
  std::string decrypt(const std::string &data);
};
} // namespace libs