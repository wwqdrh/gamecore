#pragma once

#include <cmath>
#include <iostream>
#include <optional>
#include <queue>
#include <unordered_set>
#include <vector>

namespace gamealgo {
// 地图表示：可以使用二维网格表示地图，每个单元格表示地图上的一个位置（可能是障碍物、空地或其他角色）。
// 寻路算法：A*算法用于找到从当前目标到目标位置的最优路径。
// 动态地图：每次执行路径计算时，地图状态（如障碍物、角色位置等）可能会变化，因此每一步都需要重新执行A*算法。

// 方向向量，用于表示上下左右四个方向
const std::vector<std::pair<int, int>> directions = {
    {0, 1}, {1, 0}, {0, -1}, {-1, 0}};

// 路径节点类
class Node {
public:
  int x, y;
  double g_cost, h_cost; // g_cost: 起点到当前节点的代价, h_cost: 启发式代价
                         //   Node *parent;
  Node *root_parent; // 起点的第一个邻居节点，表示下一步要走的节点

  Node(int x, int y, double g = 0, double h = 0, Node *root_parent = nullptr)
      : x(x), y(y), g_cost(g), h_cost(h), root_parent(root_parent) {}

  double f_cost() const {
    return g_cost + h_cost; // f_cost = g_cost + h_cost
  }

  bool operator==(const Node &other) const {
    return x == other.x && y == other.y;
  }
};

// 哈希函数，用于在unordered_set中比较Node对象
struct NodeHasher {
  size_t operator()(const Node &node) const {
    return std::hash<int>()(node.x) ^ std::hash<int>()(node.y);
  }
};

// 地图类
class FindWay {
public:
  int width, height;
  std::vector<std::vector<int>> grid; // 0表示空地, 1表示障碍物, 2表示其他角色

  FindWay(int width, int height)
      : width(width), height(height), grid(height, std::vector<int>(width, 0)) {
  }

  // 检查节点是否在地图内且可通行
  bool isWalkable(int x, int y) const {
    return x >= 0 && y >= 0 && x < width && y < height && grid[y][x] == 0;
  }

  // 更新地图上的某个位置
  void updatePosition(int x, int y, int state) {
    if (x >= 0 && y >= 0 && x < width && y < height) {
      grid[y][x] = state;
    }
  }

  // 打印当前地图
  void printGrid() const {
    for (const auto &row : grid) {
      for (int cell : row) {
        std::cout << cell << " ";
      }
      std::cout << std::endl;
    }
  }

  std::optional<Node> next_way(int startX, int startY, int goalX, int goalY) {
    return find_astar(startX, startY, goalX, goalY);
  }

private:
  // 曼哈顿距离作为启发式函数
  double manhattanDistance(int x1, int y1, int x2, int y2) {
    return std::abs(x1 - x2) + std::abs(y1 - y2);
  }

  std::optional<Node> find_astar(int startX, int startY, int goalX, int goalY) {
    // 优先队列，用于存储候选节点（按f_cost排序）
    auto cmp = [](const Node *a, const Node *b) {
      return a->f_cost() > b->f_cost();
    };
    std::priority_queue<Node *, std::vector<Node *>, decltype(cmp)> openList(
        cmp);
    std::unordered_set<Node, NodeHasher> closedList;

    Node *startNode = new Node(startX, startY, 0,
                               manhattanDistance(startX, startY, goalX, goalY));
    openList.push(startNode);

    int level = -1; // level为1的节点作为root_parent
    while (!openList.empty()) {
      level++;

      int curlevelnum = openList.size();
      for (int i = 0; i < curlevelnum; i++) {
        Node *currentNode = openList.top();
        openList.pop();

        // 如果找到目标节点
        if (currentNode->x == goalX && currentNode->y == goalY) {
          if (currentNode->root_parent != nullptr) {
            return *currentNode->root_parent; // 返回最终节点
          } else {
            return *currentNode;
          }
        }

        // 将当前节点添加到closedList
        closedList.insert(*currentNode);

        // 遍历当前节点的邻居
        for (const auto &direction : directions) {
          int neighborX = currentNode->x + direction.first;
          int neighborY = currentNode->y + direction.second;

          // 跳过不可通行或已访问的节点
          if (!isWalkable(neighborX, neighborY) ||
              closedList.count(Node(neighborX, neighborY))) {
            continue;
          }

          double g_cost = currentNode->g_cost + 1; // 假设所有邻居距离为1
          double h_cost = manhattanDistance(neighborX, neighborY, goalX, goalY);

          // 记录root_parent
          Node *neighborNode = nullptr;
          if (level == 1 && currentNode->root_parent == nullptr) {
            neighborNode =
                new Node(neighborX, neighborY, g_cost, h_cost, currentNode);
          } else {
            neighborNode = new Node(neighborX, neighborY, g_cost, h_cost,
                                    currentNode->root_parent);
          }

          // 将邻居节点加入openList
          openList.push(neighborNode);
        }
      }
    }

    return std::nullopt; // 未找到路径
  }
};

} // namespace gamealgo