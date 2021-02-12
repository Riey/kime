find_package(PkgConfig REQUIRED)
pkg_check_modules(GTK3 gtk+-3.0)

if(NOT DEFINED ${GTK3_FOUND})
    return()
endif()

add_library(kime-gtk3 SHARED ../gtk2/src/gtk.c ../gtk2/src/immodule.c)

target_include_directories(kime-gtk3 PRIVATE ${GTK3_INCLUDE_DIRS} ${KIME_INCLUDE})
target_link_directories(kime-gtk3 PRIVATE ${GTK3_LIBRARY_DIRS} ${KIME_LIB_DIRS})
target_link_libraries(kime-gtk3 ${GTK3_LIBRARIES} kime_engine)
