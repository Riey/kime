find_package(PkgConfig REQUIRED)
pkg_check_modules(GTK4 QUIET gtk4)

if(NOT DEFINED ${GTK4_FOUND})
    return()
endif()

add_library(kime-gtk4 SHARED src/gtk4.c ../gtk2/src/immodule.c)

target_include_directories(kime-gtk4 PRIVATE ${GTK4_INCLUDE_DIRS} ${KIME_INCLUDE})
target_link_directories(kime-gtk4 PRIVATE ${GTK4_LIBRARY_DIRS} ${KIME_LIB_DIRS})
target_link_libraries(kime-gtk4 PRIVATE ${GTK4_LIBRARIES} kime_engine)
