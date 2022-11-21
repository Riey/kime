find_package(PkgConfig REQUIRED)
pkg_check_modules(GTK3 gtk+-3.0)

if(GTK3_FOUND)
    add_library(kime-gtk3 SHARED ./src/gtk.c ./src/immodule.c ./src/str_buf.c)

    target_include_directories(kime-gtk3 PRIVATE ${GTK3_INCLUDE_DIRS} ${KIME_INCLUDE})
    target_link_directories(kime-gtk3 PRIVATE ${GTK3_LIBRARY_DIRS} ${KIME_LIB_DIRS})
    target_link_libraries(kime-gtk3 ${GTK3_LIBRARIES} ${KIME_ENGINE})
endif()
