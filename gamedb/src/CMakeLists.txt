# 收集当前目录及子目录下所有的cpp文件
file(GLOB_RECURSE SOURCES 
    "${CMAKE_CURRENT_SOURCE_DIR}/*.cpp"
)

add_library(gjson STATIC ${SOURCES})
