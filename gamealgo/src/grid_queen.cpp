
#include <cmath>
#include <iostream>
#include <vector>

#include "grid_queen.h"

namespace gamealgo {
bool GridQueen::check(std::vector<int> qs) {
  if (qs.size() != N) {
    return false;
  }
  solutions = 0;
  solve(qs, 0);
  return solutions > 0;
}

int GridQueen::checkBoard(std::vector<int> qs, int n) {
  for (int i = 0; i < n; i++) {
    // 检查同列或对角线是否有冲突
    if (qs[i] == qs[n] || abs(i - n) == abs(qs[i] - qs[n]))
      return 0;
  }
  return 1;
}

void GridQueen::solve(std::vector<int> qs, int n) {
  for (int i = 0; i < N; i++) {
    qs[n] = i;             // 尝试第n行皇后放在第i列
    if (checkBoard(qs, n)) // 检查是否冲突
    {
      if (n < N - 1)
        solve(qs, n + 1); // 递归到下一行
      else                // 找到一个解
      {
        solutions++;
      }
    }
  }
}
} // namespace gamealgo
