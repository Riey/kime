find_package(PkgConfig REQUIRED)
pkg_check_modules(GTK3 REQUIRED gtk+-3.0)

add_library(kime-gtk3 SHARED ../gtk2/src/gtk.c ../gtk2/src/immodule.c)

target_include_directories(kime-gtk3 PUBLIC ${GTK3_INCLUDE_DIRS} ${KIME_INCLUDE})
target_link_directories(kime-gtk3 PUBLIC ${GTK3_LIBRARY_DIRS} "${CMAKE_CURRENT_SOURCE_DIR}/../target/release" PUBLIC "${CMAKE_CURRENT_SOURCE_DIR}/../target/debug")
target_link_libraries(kime-gtk3 ${GTK3_LIBRARIES} kime_engine)
