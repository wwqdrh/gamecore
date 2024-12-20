#pragma once

#include "glogger.h"
#include <string>
#include <unordered_map>
#include <vector>
#include <memory>
#include <functional>

namespace gamedb {

// 资源类型枚举
enum class ResourceType {
    TEXTURE,    // 纹理
    AUDIO,      // 音频
    MODEL,      // 3D模型
    SHADER,     // 着色器
    SCRIPT,     // 脚本
    CONFIG,     // 配置文件
    OTHER       // 其他类型
};

// 资源状态
enum class ResourceState {
    UNLOADED,   // 未加载
    LOADING,    // 加载中
    LOADED,     // 已加载
    ERROR       // 加载错误
};

// 资源描述类
class ResourceDesc {
public:
    uint64_t id;                // 资源唯一ID
    std::string name;           // 资源名称
    ResourceType type;          // 资源类型
    std::string path;           // 资源文件路径
    std::string tags;           // 资源标签（用于快速查找）
    ResourceState state;        // 资源状态
    size_t size;               // 资源大小（字节）
    std::string checksum;      // 资源校验和
    std::string version;       // 资源版本号
    
    ResourceDesc() : id(0), type(ResourceType::OTHER), 
                    state(ResourceState::UNLOADED), size(0) {}
};

class GResourceManager {
public:
    using ResourceCallback = std::function<void(const ResourceDesc&)>;

    static std::shared_ptr<GResourceManager> getInstance();

    // 添加资源
    bool addResource(const ResourceDesc& resource);
    
    // 根据ID获取资源
    ResourceDesc* getResourceById(uint64_t id);
    
    // 根据名称获取资源
    ResourceDesc* getResourceByName(const std::string& name);
    
    // 根据类型获取资源列表
    std::vector<ResourceDesc*> getResourcesByType(ResourceType type);
    
    // 根据标签查询资源
    std::vector<ResourceDesc*> getResourcesByTag(const std::string& tag);
    
    // 检查资源是否存在
    bool hasResource(uint64_t id);
    
    // 移除资源
    bool removeResource(uint64_t id);
    
    // 注册资源加载回调
    void setLoadCallback(ResourceCallback callback);
    
    // 获取资源总数
    size_t getResourceCount() const;
    
    // 清除所有资源
    void clear();

private:
    GResourceManager();
    static inline std::shared_ptr<GResourceManager> instance_ = nullptr;
    
    std::unordered_map<uint64_t, ResourceDesc> resources_;
    std::unordered_map<std::string, uint64_t> name_to_id_;
    ResourceCallback load_callback_;
    
    // 生成唯一ID
    uint64_t generateUniqueId();
};

// 便捷宏定义
#define GRES_MGR gamedb::GResourceManager::getInstance()

} // namespace gamedb 