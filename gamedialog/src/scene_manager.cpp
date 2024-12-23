#include "scene_manager.h"
#include "timeline.h"

namespace gamedialog {

void SceneManager::add_timeline(const std::string& name, const std::string& data) {
    timelines[name] = std::make_shared<Timeline>(data);
}

void SceneManager::set_current_timeline(const std::string& name) {
    if (timelines.find(name) != timelines.end()) {
        current_timeline = name;
    }
}

Timeline* SceneManager::get_current_timeline() {
    if (current_timeline.empty() || timelines.find(current_timeline) == timelines.end()) {
        return nullptr;
    }
    return timelines[current_timeline].get();
}

Timeline* SceneManager::get_timeline(const std::string& name) {
    if (timelines.find(name) == timelines.end()) {
        return nullptr;
    }
    return timelines[name].get();
}

void SceneManager::set_variable(const std::string& key, const std::string& value) {
    global_variables[key] = value;
}

std::string SceneManager::get_variable(const std::string& key) const {
    auto it = global_variables.find(key);
    if (it != global_variables.end()) {
        return it->second;
    }
    return "";
}

bool SceneManager::has_variable(const std::string& key) const {
    return global_variables.find(key) != global_variables.end();
}

} // namespace gamedialog 