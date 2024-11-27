#include <iostream>
#include <vector>

namespace gamealgo {
// 结构体，定义地图上某点的行、列序号
struct Location {
  int i;
  int j;
};

// 地图类，负责管理地图数据及相关操作
class Map {
protected:
  int row_num;
  int col_num;

public:
  Map() : Map(5, 6) {}
  Map(int row, int col) : row_num(row), col_num(col) {
    // 初始化地图，1表示可走，0表示不可走
    map = std::vector<std::vector<int>>(row_num, std::vector<int>(col_num, 1));
    goalLocNum = row_num * col_num;
    startLoc = {0, 0};
  }
  int get_row() const { return row_num; }
  int get_col() const { return col_num; }
  void set_start(int i, int j) { startLoc = {i, j}; }
  void set_wall(int i, int j) {
    if (i >= 0 && i < row_num && j >= 0 && j < col_num) {
      map[i][j] = 0;
      goalLocNum -= 1;
    }
  }

  bool isAccessible(int i, int j) const {
    return i >= 0 && i < row_num && j >= 0 && j < col_num && map[i][j] == 1;
  }

  int getGoalLocNum() const { return goalLocNum; }

  Location getStartLocation() const { return startLoc; }

private:
  std::vector<std::vector<int>> map; // 地图数据
  Location startLoc;                 // 起始位置
  int goalLocNum;                    // 目标位置的个数
};

// 游戏类，负责管理路径查找
class OneStrokeGame {
public:
  OneStrokeGame(Map &map) : gameMap(map) {
    // 初始化路径和方向
    path.resize(map.get_row() * map.get_col(), {-1, -1});
    directions = {{-1, 0}, {1, 0}, {0, -1}, {0, 1}};
    pathDirs.resize(map.get_row() * map.get_col(), 0);
    path[0] = gameMap.getStartLocation(); // 将起始位置放入路径
  }

  void findPath() { findNext(path, pathDirs); }

  void printPath() const {
    for (const auto &loc : path) {
      if (loc.i != -1 && loc.j != -1) {
        std::cout << "(" << loc.i << ", " << loc.j << ") ";
      }
    }
    std::cout << std::endl;
  }

private:
  Map &gameMap;
  std::vector<Location> path;               // 记录路径
  std::vector<int> pathDirs;                // 记录路径的方向
  std::vector<std::vector<int>> directions; // 上下左右四个方向

  bool isNeighbour(const Location &loc1, const Location &loc2) const {
    for (const auto &dir : directions) {
      if (loc1.i + dir[0] == loc2.i && loc1.j + dir[1] == loc2.j) {
        return true;
      }
    }
    return false;
  }

  bool isInPath(const Location &loc) const {
    for (const auto &p : path) {
      if (p.i == loc.i && p.j == loc.j) {
        return true;
      }
    }
    return false;
  }

  void findNext(std::vector<Location> &pa, std::vector<int> &dirs) {
    Location lastLoc;
    int totalNum;
    updateParameters(pa, lastLoc, totalNum);

    if (totalNum == gameMap.getGoalLocNum()) {
      return; // 找到解
    }

    if (dirs[totalNum - 1] >= 4) { // 如果方向序号大于等于4，回溯
      dirs[totalNum - 1] = 0;
      pa[totalNum - 1] = {-1, -1};
      totalNum--;
      dirs[totalNum - 1]++;
      findNext(pa, dirs);
    } else {
      Location next = {lastLoc.i + directions[dirs[totalNum - 1]][0],
                       lastLoc.j + directions[dirs[totalNum - 1]][1]};
      if (gameMap.isAccessible(next.i, next.j) && !isInPath(next)) {
        pa[totalNum] = next;
        dirs[totalNum] = 0;
        totalNum++;
        findNext(pa, dirs);
      } else {
        dirs[totalNum - 1]++;
        findNext(pa, dirs);
      }
    }
  }

  void updateParameters(const std::vector<Location> &pa, Location &lastLoc,
                        int &totalNum) const {
    totalNum = 0;
    while (pa[totalNum].i != -1 && pa[totalNum].j != -1) {
      totalNum++;
    }
    lastLoc = pa[totalNum - 1];
  }
};

} // namespace algo