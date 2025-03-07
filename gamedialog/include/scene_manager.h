#pragma once

#include <map>
#include <memory>
#include <string>
#include <unordered_map>

namespace gamedialog {

class Timeline; // 前向声明

class SceneManager {
public:
    static SceneManager& instance() {
        static SceneManager instance;
        return instance;
    }

    // 禁用拷贝和赋值
    SceneManager(const SceneManager&) = delete;
    SceneManager& operator=(const SceneManager&) = delete;

    // Timeline管理
    void add_timeline(const std::string& name, const std::string& data);
    void set_current_timeline(const std::string& name);
    Timeline* get_current_timeline();
    Timeline* get_timeline(const std::string& name);

    // 全局变量管理
    std::vector<std::string> get_all_variables() const {
        std::vector<std::string> variables;
        for (const auto& pair : global_variables) {
            variables.push_back(pair.first + "=" + pair.second);
        }
        return variables;
    }
    void set_variable(const std::string& key, const std::string& value);
    std::string get_variable(const std::string& key) const;
    bool has_variable(const std::string& key) const;

    // Add new method
    std::map<std::string, std::vector<std::string>> get_all_available_stages() const;

private:
    SceneManager() = default;
    
    std::unordered_map<std::string, std::shared_ptr<Timeline>> timelines;
    std::string current_timeline;
    std::unordered_map<std::string, std::string> global_variables;
};

} // namespace gamedialog 