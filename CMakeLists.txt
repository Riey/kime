cmake_minimum_required(VERSION 3.1.0)

project(kime VERSION 0.8.0)

set(KIME_INCLUDE ${CMAKE_CURRENT_SOURCE_DIR}/engine/cffi)

include(GNUInstallDirs)

add_subdirectory(gtk3)
add_subdirectory(qt5)
