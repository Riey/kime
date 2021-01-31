set(CMAKE_CXX_STANDARD 20)

set(CMAKE_AUTOMOC ON)
set(CMAKE_AUTORCC ON)
set(CMAKE_AUTOUIC ON)

find_package(Qt6 6.0.0 COMPONENTS Gui REQUIRED)

add_library(kime-qt6 SHARED ../qt5/src/plugin.cc ../qt5/src/input_context.cc)

target_include_directories(kime-qt6 PUBLIC ${Qt6Gui_PRIVATE_INCLUDE_DIRS} ${KIME_INCLUDE})
target_link_directories(kime-qt6 PUBLIC "${CMAKE_CURRENT_SOURCE_DIR}/../target/release" PUBLIC "${CMAKE_CURRENT_SOURCE_DIR}/../target/debug")
target_link_libraries(kime-qt6 PRIVATE kime_engine Qt6::Gui)
