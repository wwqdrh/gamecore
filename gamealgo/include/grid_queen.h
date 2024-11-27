#include <cmath>
#include <iostream>
#include <vector>

namespace gamealgo {
class GridQueen {
protected:
  int N;
  int solutions;

protected:
  void solve(std::vector<int> qs, int n);
  int checkBoard(std::vector<int> qs, int n);

public:
  GridQueen() : N(8), solutions(0) {}
  GridQueen(int n) : N(n), solutions(0) {}
  bool check(std::vector<int> qs);
};
} // namespace gamealgo