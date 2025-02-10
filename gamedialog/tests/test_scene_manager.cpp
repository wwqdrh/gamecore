#include "scene_manager.h"
#include <gtest/gtest.h>

using namespace gamedialog;

class SceneManagerTest : public ::testing::Test {
protected:
    void SetUp() override {
        // 在每个测试前重置一些测试数据
        auto& manager = SceneManager::instance();
        manager.add_timeline("test_scene", "[scene1]\nHello World");
    }
};

TEST_F(SceneManagerTest, SingletonInstance) {
    auto& manager1 = SceneManager::instance();
    auto& manager2 = SceneManager::instance();
    EXPECT_EQ(&manager1, &manager2);
}

TEST_F(SceneManagerTest, TimelineManagement) {
    auto& manager = SceneManager::instance();
    
    // 测试获取timeline
    EXPECT_NE(manager.get_timeline("test_scene"), nullptr);
    EXPECT_EQ(manager.get_timeline("non_existent"), nullptr);

    // 测试设置和获取当前timeline
    manager.set_current_timeline("test_scene");
    EXPECT_NE(manager.get_current_timeline(), nullptr);
    
    manager.set_current_timeline("non_existent");
    EXPECT_NE(manager.get_current_timeline(), nullptr);
}

TEST_F(SceneManagerTest, GlobalVariables) {
    auto& manager = SceneManager::instance();
    
    // 测试设置和获取变量
    manager.set_variable("test_key", "test_value");
    EXPECT_EQ(manager.get_variable("test_key"), "test_value");
    EXPECT_TRUE(manager.has_variable("test_key"));
    
    // 测试获取不存在的变量
    EXPECT_EQ(manager.get_variable("non_existent"), "");
    EXPECT_FALSE(manager.has_variable("non_existent"));
} 