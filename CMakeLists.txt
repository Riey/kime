cmake_minimum_required(VERSION 3.1.0)

project(kime VERSION 0.8.1)

option(ENABLE_GTK3 "Enable GTK3 immodule" OFF)
option(ENABLE_QT5 "Enable Qt5 immodule" OFF)

set(CMAKE_SKIP_RPATH TRUE)
set(CMAKE_C_FLAGS_DEBUG "${CMAKE_C_FLAGS_DEBUG} -DDEBUG")
set(CMAKE_CXX_FLAGS_DEBUG "${CMAKE_CXX_FLAGS_DEBUG} -DDEBUG")
set(CMAKE_LIBRARY_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/lib)
set(CMAKE_RUNTIME_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/bin)
set(KIME_INCLUDE ${CMAKE_CURRENT_SOURCE_DIR}/engine/cffi)

include(GNUInstallDirs)

add_subdirectory(gtk2)
if(${ENABLE_GTK3})
    add_subdirectory(gtk3)
endif()

if(${ENABLE_QT5})
    add_subdirectory(qt5)
endif()
