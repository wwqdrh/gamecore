#include <fstream>
#include <stdexcept>

#include "store.h"

namespace gamedb {
void FileStore::saveData(const std::string &data) {
  std::string encryptedData = encrypt(data);
  if (customSaveHandler_) {
    // 使用自定义保存处理函数
    customSaveHandler_(encryptedData);
    return;
  }
  std::ofstream file(filename_);
  if (!file) {
    return;
    // throw std::runtime_error("Unable to open file for writing");
  }
  file << encryptedData;
}

std::string FileStore::loadData() {
  std::string data = "";
  if (customLoadHandler_) {
    // 使用自定义读取处理函数
    data = customLoadHandler_();
  } else {
    std::ifstream file(filename_);
    if (!file) {
      return "";
    }

    data = std::string((std::istreambuf_iterator<char>(file)),
                       std::istreambuf_iterator<char>());
  }
  std::string decryptedData = decrypt(data);

  return decryptedData;
}

std::string FileStore::encrypt(const std::string &data) const {
  // 简单的异或加密，使用固定的密钥
  const char key = 0x42;
  std::string result = data;
  for (char &c : result) {
    c ^= key;
  }
  return result;
}

std::string FileStore::decrypt(const std::string &data) const {
  // 解密过程与加密相同（异或运算的特性）
  return encrypt(data);
}
} // namespace gamedb