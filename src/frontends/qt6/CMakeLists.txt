set(CMAKE_CXX_STANDARD 20)

set(CMAKE_AUTOMOC ON)
set(CMAKE_AUTORCC ON)
set(CMAKE_AUTOUIC ON)

find_package(Qt6 6.0.0 QUIET COMPONENTS Gui GuiPrivate QUIET)
find_package(Qt5 5.1.0 QUIET COMPONENTS Gui QUIET)

if(NOT Qt6_FOUND)
    return()
endif()

add_library(kime-qt6 SHARED ../qt5/src/plugin.cc ../qt5/src/input_context.cc)

target_compile_definitions(kime-qt6 PRIVATE KIME_QT_IID="org.qt-project.Qt.QPlatformInputContextFactoryInterface.5.1")
target_include_directories(kime-qt6 PRIVATE ${KIME_INCLUDE})
target_link_directories(kime-qt6 PRIVATE ${KIME_LIB_DIRS})
target_link_libraries(kime-qt6 PRIVATE ${KIME_ENGINE} Qt6::Gui Qt6::GuiPrivate)