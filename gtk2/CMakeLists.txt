find_package(PkgConfig REQUIRED)
pkg_check_modules(GTK2 REQUIRED gtk+-2.0)

add_library(kime-gtk2 SHARED src/gtk.c src/immodule.c)

target_compile_definitions(kime-gtk2 PRIVATE GTK_DISABLE_DEPRECATED)
target_include_directories(kime-gtk2 PUBLIC ${GTK2_INCLUDE_DIRS} ${KIME_INCLUDE})
target_link_directories(kime-gtk2 PUBLIC ${GTK2_LIBRARY_DIRS} "${CMAKE_CURRENT_SOURCE_DIR}/../target/release" PUBLIC "${CMAKE_CURRENT_SOURCE_DIR}/../target/debug")
target_link_libraries(kime-gtk2 PUBLIC ${GTK2_LIBRARIRES} kime_engine)
