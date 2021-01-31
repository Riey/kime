set(CMAKE_CXX_STANDARD 20)

set(CMAKE_AUTOMOC ON)
set(CMAKE_AUTORCC ON)
set(CMAKE_AUTOUIC ON)

find_package(Qt5 5.1.0 COMPONENTS Gui REQUIRED)

add_library(kime-qt5 SHARED src/plugin.cc src/input_context.cc)

target_include_directories(kime-qt5 PRIVATE ${Qt5Gui_PRIVATE_INCLUDE_DIRS} ${KIME_INCLUDE})
target_link_directories(kime-qt5 PRIVATE ${KIME_LIB_DIRS})
target_link_libraries(kime-qt5 PRIVATE kime_engine Qt5::Gui)
