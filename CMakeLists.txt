cmake_minimum_required(VERSION 3.1.0)

project(kime VERSION 0.8.0)

set(CMAKE_SKIP_RPATH TRUE)
set(CMAKE_C_FLAGS_DEBUG "${CMAKE_C_FLAGS_DEBUG} -DDEBUG")
set(CMAKE_CXX_FLAGS_DEBUG "${CMAKE_CXX_FLAGS_DEBUG} -DDEBUG")
set(KIME_INCLUDE ${CMAKE_CURRENT_SOURCE_DIR}/engine/cffi)

include(GNUInstallDirs)

add_subdirectory(gtk3)
add_subdirectory(qt5)
