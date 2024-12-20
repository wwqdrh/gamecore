#include "gresource_manager.h"
#include <algorithm>
#include <chrono>

namespace gamedb {

GResourceManager::GResourceManager() {
    GLOG_INFO("ResourceManager", "Resource manager initialized");
}

std::shared_ptr<GResourceManager> GResourceManager::getInstance() {
    if (!instance_) {
        instance_ = std::shared_ptr<GResourceManager>(new GResourceManager());
    }
    return instance_;
}

bool GResourceManager::addResource(const ResourceDesc& resource) {
    if (resource.id == 0) {
        GLOG_ERROR("ResourceManager", "Invalid resource ID: 0");
        return false;
    }
    
    if (resources_.find(resource.id) != resources_.end()) {
        GLOG_WARNING("ResourceManager", "Resource ID " + std::to_string(resource.id) + " already exists");
        return false;
    }
    
    if (name_to_id_.find(resource.name) != name_to_id_.end()) {
        GLOG_WARNING("ResourceManager", "Resource name '" + resource.name + "' already exists");
        return false;
    }
    
    resources_[resource.id] = resource;
    name_to_id_[resource.name] = resource.id;
    
    GLOG_INFO("ResourceManager", "Added resource: " + resource.name + " (ID: " + 
              std::to_string(resource.id) + ")");
    
    if (load_callback_) {
        load_callback_(resource);
    }
    
    return true;
}

ResourceDesc* GResourceManager::getResourceById(uint64_t id) {
    auto it = resources_.find(id);
    if (it != resources_.end()) {
        return &it->second;
    }
    GLOG_DEBUG("ResourceManager", "Resource ID " + std::to_string(id) + " not found");
    return nullptr;
}

ResourceDesc* GResourceManager::getResourceByName(const std::string& name) {
    auto it = name_to_id_.find(name);
    if (it != name_to_id_.end()) {
        return getResourceById(it->second);
    }
    GLOG_DEBUG("ResourceManager", "Resource name '" + name + "' not found");
    return nullptr;
}

std::vector<ResourceDesc*> GResourceManager::getResourcesByType(ResourceType type) {
    std::vector<ResourceDesc*> result;
    for (auto& pair : resources_) {
        if (pair.second.type == type) {
            result.push_back(&pair.second);
        }
    }
    GLOG_DEBUG("ResourceManager", "Found " + std::to_string(result.size()) + 
               " resources of specified type");
    return result;
}

std::vector<ResourceDesc*> GResourceManager::getResourcesByTag(const std::string& tag) {
    std::vector<ResourceDesc*> result;
    for (auto& pair : resources_) {
        if (pair.second.tags.find(tag) != std::string::npos) {
            result.push_back(&pair.second);
        }
    }
    GLOG_DEBUG("ResourceManager", "Found " + std::to_string(result.size()) + 
               " resources with tag: " + tag);
    return result;
}

bool GResourceManager::hasResource(uint64_t id) {
    return resources_.find(id) != resources_.end();
}

bool GResourceManager::removeResource(uint64_t id) {
    auto it = resources_.find(id);
    if (it != resources_.end()) {
        name_to_id_.erase(it->second.name);
        resources_.erase(it);
        GLOG_INFO("ResourceManager", "Removed resource ID: " + std::to_string(id));
        return true;
    }
    GLOG_WARNING("ResourceManager", "Failed to remove resource ID: " + std::to_string(id));
    return false;
}

void GResourceManager::setLoadCallback(ResourceCallback callback) {
    load_callback_ = callback;
}

size_t GResourceManager::getResourceCount() const {
    return resources_.size();
}

void GResourceManager::clear() {
    resources_.clear();
    name_to_id_.clear();
    GLOG_INFO("ResourceManager", "All resources cleared");
}

uint64_t GResourceManager::generateUniqueId() {
    // 使用时间戳和资源数量生成唯一ID
    auto now = std::chrono::system_clock::now();
    auto duration = now.time_since_epoch();
    auto millis = std::chrono::duration_cast<std::chrono::milliseconds>(duration).count();
    return static_cast<uint64_t>(millis) + resources_.size();
}

} // namespace gamedb 