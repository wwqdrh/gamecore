cmake_minimum_required(VERSION 3.6)
project(gamedb)

# 构建选项
set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)


# 检测是否是 WASM 构建
if(EMSCRIPTEN)
    # WASM 特定设置
    set(CMAKE_EXECUTABLE_SUFFIX ".js")
    
    # WASM 编译选项
    set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -s WASM=1")
    set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -s ALLOW_MEMORY_GROWTH=1")
    set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -s NO_EXIT_RUNTIME=1")
    
    # 如果需要使用 pthread
    # set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -s USE_PTHREADS=1")
    
    # 如果需要导出特定函数
    # set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -s EXPORTED_FUNCTIONS=['_main']")
    
    # 设置内存初始值和最大值（根据需要调整）
    set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -s INITIAL_MEMORY=16MB -s MAXIMUM_MEMORY=512MB")
    
    # 链接时优化
    set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -O3")
endif()

# 如果没有设置ARCH那么就设置为x86_64
if(NOT DEFINED ARCH)
    if(EMSCRIPTEN)
        set(ARCH wasm32)
    else()
        set(ARCH x86_64)
    endif()
endif()

# 如果没有设置CMAKE_BUILD_TYPE那么就是Debug
if(NOT DEFINED CMAKE_BUILD_TYPE)
    set(CMAKE_BUILD_TYPE Debug)
endif()

include_directories("include")
include_directories("3rd/rapidjson-master")

add_subdirectory(src)

if(CMAKE_BUILD_TYPE STREQUAL "Debug")
    message(STATUS "Building Debug version, enabling test coverage")

    # 设置覆盖率编译选项
    set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -O0 -Wall -g --coverage -fprofile-arcs -ftest-coverage")
    set(CMAKE_EXE_LINKER_FLAGS "${CMAKE_EXE_LINKER_FLAGS} --coverage")


    # 添加gcovr配置
    find_program(GCOVR_PATH gcovr REQUIRED)
    
    # 添加生成覆盖率报告的自定义目标
    add_custom_target(coverage
        # 清理旧的覆盖率数据
        COMMAND ${CMAKE_COMMAND} -E remove_directory ${CMAKE_BINARY_DIR}/coverage
        COMMAND ${CMAKE_COMMAND} -E make_directory ${CMAKE_BINARY_DIR}/coverage
        
        # 确保先运行所有测试
        COMMAND ${CMAKE_COMMAND} --build ${CMAKE_BINARY_DIR} --target check

        # 生成覆盖率报告
        # 创建覆盖率报告目录
        COMMAND ${CMAKE_COMMAND} -E make_directory ${CMAKE_BINARY_DIR}/coverage
        
        # 生成HTML格式的详细报告
        COMMAND ${GCOVR_PATH}
            --root ${CMAKE_SOURCE_DIR}
            --object-directory ${CMAKE_BINARY_DIR}
            --output ${CMAKE_BINARY_DIR}/coverage/coverage.html
            --html --html-details
            --exclude-throw-branches
            --exclude ".*3rd/.*"
            --exclude ".*test_.*"
            --exclude ".*gtest.*"
            --exclude ".*gmock.*"
            
        # 生成XML格式报告(用于CI集成)
        COMMAND ${GCOVR_PATH}
            --root ${CMAKE_SOURCE_DIR}
            --object-directory ${CMAKE_BINARY_DIR}
            --output ${CMAKE_BINARY_DIR}/coverage/coverage.xml
            --xml
            --exclude-throw-branches
            --exclude ".*3rd/.*"
            --exclude ".*test_.*"
            --exclude ".*gtest.*"
            --exclude ".*gmock.*"
            
        # 生成简单的控制台摘要
        COMMAND ${GCOVR_PATH}
            --root ${CMAKE_SOURCE_DIR}
            --object-directory ${CMAKE_BINARY_DIR}
            --exclude-throw-branches
            --exclude ".*3rd/.*"
            --exclude ".*test_.*"
            --exclude ".*gtest.*"
            --exclude ".*gmock.*"
            
        WORKING_DIRECTORY ${CMAKE_BINARY_DIR}
        COMMENT "Generating code coverage reports"
    )

    include(FetchContent)
    FetchContent_Declare(
        googletest
        URL https://github.com/google/googletest/releases/download/v1.15.2/googletest-1.15.2.tar.gz
        DOWNLOAD_EXTRACT_TIMESTAMP TRUE
    )
    # For Windows: Prevent overriding the parent project's compiler/linker settings
    set(gtest_force_shared_crt ON CACHE BOOL "" FORCE)
    FetchContent_MakeAvailable(googletest)

    # 启用测试
    enable_testing()
    include(GoogleTest)

    # 设置测试选项
    option(TEST_OUTPUT_ON_FAILURE "Show output when tests fail" ON)

    if(TEST_OUTPUT_ON_FAILURE)
        set(CMAKE_CTEST_ARGUMENTS "--output-on-failure")
    endif()

    # 查找所有的测试文件
    file(GLOB TEST_SOURCES "tests/test_*.cpp")
    if(NOT TEST_SOURCES)
        message(WARNING "No test files found matching pattern '${TEST_DIR}/test_*.cpp'")
    endif()

    # 存储所有测试目标的列表
    set(TEST_TARGETS "")
    
    # 为每个测试文件创建一个测试可执行文件
    foreach(TEST_FILE ${TEST_SOURCES})
        get_filename_component(TEST_NAME ${TEST_FILE} NAME_WE)
        add_executable(${TEST_NAME} ${TEST_FILE})
        
        target_link_libraries(${TEST_NAME} PRIVATE 
            GTest::gtest 
            GTest::gtest_main
            gjson
        )
        
        add_test(NAME ${TEST_NAME} COMMAND ${TEST_NAME})
        gtest_discover_tests(${TEST_NAME}
            WORKING_DIRECTORY ${CMAKE_CURRENT_BINARY_DIR}
            PROPERTIES TIMEOUT 10
        )
        list(APPEND TEST_TARGETS ${TEST_NAME})
    endforeach()
    # 添加一个运行所有测试的目标
    add_custom_target(check 
        COMMAND ${CMAKE_CTEST_COMMAND} --output-on-failure
        DEPENDS ${TEST_TARGETS}
        WORKING_DIRECTORY ${CMAKE_BINARY_DIR}
        COMMENT "Running all tests..."  
    )
else()
    message(STATUS "Building Release version, skipping test coverage")
endif()