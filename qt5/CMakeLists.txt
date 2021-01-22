cmake_minimum_required(VERSION 3.1.0)

project(kime-qt5 VERSION 0.8.0 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 20)

set(CMAKE_AUTOMOC ON)
set(CMAKE_AUTORCC ON)
set(CMAKE_AUTOUIC ON)

find_package(Qt5Gui)

include_directories(${Qt5Gui_INCLUDE_DIRS})
include_directories(${Qt5Gui_PRIVATE_INCLUDE_DIRS})

add_definitions(${Qt5Gui_DEFINITIONS})

set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS}")

add_library(kime-qt5 SHARED src/plugin.cc)

target_link_libraries(kime-qt5 kime_engine)
