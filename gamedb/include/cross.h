#pragma once

#ifdef __ANDROID__
// 针对 Android 平台的特定头文件
#include <cstdint> // 用于 uint8_t
#include <vector>  // 用于 std::vector
#elif defined(_WIN32) || defined(_WIN64)
// 针对 Windows 平台的特定头文件
#include <cstdint> // 用于 uint8_t
#include <vector>  // 用于 std::vector
#include <numeric>
#else
// 其他平台（例如 Linux、Mac）通用头文件
#include <cstdint> // 用于 uint8_t
#include <vector>  // 用于 std::vector
#endif