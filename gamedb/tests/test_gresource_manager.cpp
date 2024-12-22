#include "gresource_manager.h"
#include <gtest/gtest.h>

using namespace gamedb;

class GResourceManagerTest : public ::testing::Test {
protected:
    void SetUp() override {
        manager = GResourceManager::getInstance();
        manager->clear();
    }

    void TearDown() override {
        manager->clear();
    }

    std::shared_ptr<GResourceManager> manager;

    // 辅助函数：创建测试资源
    ResourceDesc createTestResource(uint64_t id, const std::string& name, ResourceType type) {
        ResourceDesc res;
        res.id = id;
        res.name = name;
        res.type = type;
        res.path = "test/path/" + name;
        res.tags = "test,unit-test";
        res.state = ResourceState::UNLOADED;
        res.size = 1024;
        res.checksum = "test-checksum";
        res.version = "1.0";
        return res;
    }
};

// 测试添加资源
TEST_F(GResourceManagerTest, AddResource) {
    auto res = createTestResource(1, "test_texture", ResourceType::TEXTURE);
    EXPECT_TRUE(manager->addResource(res));
    EXPECT_EQ(manager->getResourceCount(), 1);
    
    // 测试添加重复ID
    auto res2 = createTestResource(1, "test_texture2", ResourceType::TEXTURE);
    EXPECT_FALSE(manager->addResource(res2));
    
    // 测试添加重复名称
    auto res3 = createTestResource(2, "test_texture", ResourceType::TEXTURE);
    EXPECT_FALSE(manager->addResource(res3));
    
    // 测试添加ID为0的资源
    auto res4 = createTestResource(0, "invalid_resource", ResourceType::TEXTURE);
    EXPECT_FALSE(manager->addResource(res4));
}

// 测试通过ID获取资源
TEST_F(GResourceManagerTest, GetResourceById) {
    auto res = createTestResource(1, "test_texture", ResourceType::TEXTURE);
    manager->addResource(res);
    
    auto found = manager->getResourceById(1);
    ASSERT_NE(found, nullptr);
    EXPECT_EQ(found->name, "test_texture");
    EXPECT_EQ(found->type, ResourceType::TEXTURE);
    
    // 测试获取不存在的资源
    EXPECT_EQ(manager->getResourceById(999), nullptr);
}

// 测试通过名称获取资源
TEST_F(GResourceManagerTest, GetResourceByName) {
    auto res = createTestResource(1, "test_texture", ResourceType::TEXTURE);
    manager->addResource(res);
    
    auto found = manager->getResourceByName("test_texture");
    ASSERT_NE(found, nullptr);
    EXPECT_EQ(found->id, 1);
    EXPECT_EQ(found->type, ResourceType::TEXTURE);
    
    // 测试获取不存在的资源
    EXPECT_EQ(manager->getResourceByName("nonexistent"), nullptr);
}

// 测试通过类型获取资源
TEST_F(GResourceManagerTest, GetResourcesByType) {
    manager->addResource(createTestResource(1, "texture1", ResourceType::TEXTURE));
    manager->addResource(createTestResource(2, "texture2", ResourceType::TEXTURE));
    manager->addResource(createTestResource(3, "audio1", ResourceType::AUDIO));
    
    auto textures = manager->getResourcesByType(ResourceType::TEXTURE);
    EXPECT_EQ(textures.size(), 2);
    
    auto audio = manager->getResourcesByType(ResourceType::AUDIO);
    EXPECT_EQ(audio.size(), 1);
    
    auto models = manager->getResourcesByType(ResourceType::MODEL);
    EXPECT_EQ(models.size(), 0);
}

// 测试通过标签获取资源
TEST_F(GResourceManagerTest, GetResourcesByTag) {
    auto res1 = createTestResource(1, "player_texture", ResourceType::TEXTURE);
    res1.tags = "player,character,texture";
    manager->addResource(res1);
    
    auto res2 = createTestResource(2, "enemy_texture", ResourceType::TEXTURE);
    res2.tags = "enemy,character,texture";
    manager->addResource(res2);
    
    auto characterRes = manager->getResourcesByTag("character");
    EXPECT_EQ(characterRes.size(), 2);
    
    auto playerRes = manager->getResourcesByTag("player");
    EXPECT_EQ(playerRes.size(), 1);
    
    auto nonexistentRes = manager->getResourcesByTag("nonexistent");
    EXPECT_EQ(nonexistentRes.size(), 0);
}

// 测试资源移除
TEST_F(GResourceManagerTest, RemoveResource) {
    auto res = createTestResource(1, "test_texture", ResourceType::TEXTURE);
    manager->addResource(res);
    
    EXPECT_TRUE(manager->hasResource(1));
    EXPECT_TRUE(manager->removeResource(1));
    EXPECT_FALSE(manager->hasResource(1));
    EXPECT_EQ(manager->getResourceCount(), 0);
    
    // 测试移除不存在的资源
    EXPECT_FALSE(manager->removeResource(999));
}

// 测试资源加载回调
TEST_F(GResourceManagerTest, LoadCallback) {
    bool callbackCalled = false;
    std::string loadedResourceName;
    
    manager->setLoadCallback([&](const ResourceDesc& res) {
        callbackCalled = true;
        loadedResourceName = res.name;
    });
    
    auto res = createTestResource(1, "test_texture", ResourceType::TEXTURE);
    manager->addResource(res);
    
    EXPECT_TRUE(callbackCalled);
    EXPECT_EQ(loadedResourceName, "test_texture");
}

// 测试清除所有资源
TEST_F(GResourceManagerTest, Clear) {
    manager->addResource(createTestResource(1, "res1", ResourceType::TEXTURE));
    manager->addResource(createTestResource(2, "res2", ResourceType::AUDIO));
    
    EXPECT_EQ(manager->getResourceCount(), 2);
    manager->clear();
    EXPECT_EQ(manager->getResourceCount(), 0);
} 