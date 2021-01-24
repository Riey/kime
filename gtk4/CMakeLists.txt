find_package(PkgConfig REQUIRED)
pkg_check_modules(GTK4 REQUIRED gtk4)

add_library(kime-gtk4 SHARED src/gtk4.c ../gtk2/src/immodule.c)

target_include_directories(kime-gtk4 PUBLIC ${GTK4_INCLUDE_DIRS} ${KIME_INCLUDE})
target_link_directories(kime-gtk4 PUBLIC ${GTK4_LIBRARY_DIRS} "${CMAKE_CURRENT_SOURCE_DIR}/../target/release" PUBLIC "${CMAKE_CURRENT_SOURCE_DIR}/../target/debug")
target_link_libraries(kime-gtk4 PUBLIC ${GTK4_LIBRARIRES} kime_engine)
