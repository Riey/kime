set(CMAKE_CXX_STANDARD 20)

set(CMAKE_AUTOMOC ON)
set(CMAKE_AUTORCC ON)
set(CMAKE_AUTOUIC ON)

find_package(Qt5Gui 5.1.0 REQUIRED Private)

add_definitions(${Qt5Gui_DEFINITIONS})

add_library(kime-qt5 SHARED src/plugin.cc src/input_context.cc)

target_include_directories(kime-qt5 PUBLIC ${Qt5Gui_INCLUDE_DIRS} ${Qt5Gui_PRIVATE_INCLUDE_DIRS} ${KIME_INCLUDE})
target_link_directories(kime-qt5 PUBLIC "${CMAKE_CURRENT_SOURCE_DIR}/../target/release" PUBLIC "${CMAKE_CURRENT_SOURCE_DIR}/../target/debug")
target_link_libraries(kime-qt5 PUBLIC kime_engine PUBLIC ${Qt5Gui_LIBRARIES})
