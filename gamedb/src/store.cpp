#include <fstream>
#include <stdexcept>

#include "store.h"

namespace gamedb {
void FileStore::saveData(const std::string &data) {
  std::vector<uint8_t> encryptedData = encrypt(data);
  if (customSaveHandler_ != nullptr) {
    // 使用自定义保存处理函数
    customSaveHandler_(encryptedData);
    return;
  }
  std::ofstream file(filename_, std::ios::binary);
  if (!file) {
    return;
    // throw std::runtime_error("Unable to open file for writing");
  }
  file.write(reinterpret_cast<const char *>(encryptedData.data()),
             encryptedData.size());
}

std::string FileStore::loadData() {
  std::vector<uint8_t> data;
  if (customLoadHandler_ != nullptr) {
    // 使用自定义读取处理函数
    data = customLoadHandler_();
  } else {
    std::ifstream file(filename_, std::ios::binary);
    if (!file) {
      return "";
    }
    data = std::vector<uint8_t>((std::istreambuf_iterator<char>(file)),
                                (std::istreambuf_iterator<char>()));
  }

  return decrypt(data);
}

std::vector<uint8_t> FileStore::encrypt(const std::string &data) const {
  // 使用固定的密钥进行异或加密
  const char key = 0x42;
  std::vector<uint8_t> result(data.begin(), data.end());

  if (is_encrypt_) {
    for (auto &c : result) {
      c ^= key;
    }
  }

  return result;
}

std::string FileStore::decrypt(const std::vector<uint8_t> &data) const {
  // 解密过程与加密相同（异或运算的特性）
  std::vector<uint8_t> decryptedData = data;

  if (is_encrypt_) {
    const char key = 0x42;
    for (auto &c : decryptedData) {
      c ^= key;
    }
  }

  // 将解密后的数据转换为字符串
  return std::string(decryptedData.begin(), decryptedData.end());
}
} // namespace gamedb