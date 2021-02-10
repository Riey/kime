find_package(PkgConfig REQUIRED)
pkg_check_modules(GTK2 gtk+-2.0)

if(${GTK2_FOUND} STREQUAL "0")
    return()
endif()

add_library(kime-gtk2 SHARED src/gtk.c src/immodule.c)

target_compile_definitions(kime-gtk2 PRIVATE GTK_DISABLE_DEPRECATED)
target_include_directories(kime-gtk2 PRIVATE ${GTK2_INCLUDE_DIRS} ${KIME_INCLUDE})
target_link_directories(kime-gtk2 PRIVATE ${GTK2_LIBRARY_DIRS} ${KIME_LIB_DIRS})
target_link_libraries(kime-gtk2 PRIVATE ${GTK2_LIBRARIES} kime_engine)
