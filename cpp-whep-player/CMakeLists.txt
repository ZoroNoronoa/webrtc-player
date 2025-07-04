cmake_minimum_required(VERSION 3.16.0)
project(whep-play)

set(CMAKE_CXX_STANDARD 17)

set(CMAKE_CXX_FLAGS_DEBUG "-ggdb -O0 -DDEBUG -Wall -Werror -Wuninitialized -Wsign-compare -Wno-missing-braces")
set(CMAKE_CXX_FLAGS_RELEASE "-ggdb -O3 -DNDEBUG -Wall -Werror -Wuninitialized -Wsign-compare -Wno-missing-braces")

if (APPLE)
        message("OSX")
        # set(ENV{PKG_CONFIG_PATH} "/usr/local/Cellar/icu4c/71.1/lib/pkgconfig:/usr/local/Cellar/libsoup@2/2.74.2_1/lib/pkgconfig:/opt/homebrew/opt/libsoup@2/lib/pkgconfig:/opt/homebrew/opt/icu4c/lib/pkgconfig/")
        # 查找不到, 替换成:
        # export PKG_CONFIG_PATH=$(find /usr/local/Cellar -name 'pkgconfig' -type d | grep lib/pkgconfig | tr '\n' ':' | sed s/.$//)
endif()

find_package(PkgConfig)
pkg_search_module(GLIB REQUIRED glib-2.0)
pkg_check_modules(GSTREAMER REQUIRED gstreamer-1.0)
pkg_check_modules(GSTREAMER_WEBRTC REQUIRED gstreamer-webrtc-1.0)
pkg_check_modules(GSTREAMER_SDP REQUIRED gstreamer-sdp-1.0)
pkg_check_modules(SOUP REQUIRED libsoup-2.4)

set(FILES
        src/main.cc
)

add_executable(whep-play ${FILES})

target_include_directories(${PROJECT_NAME} PRIVATE
        ${PROJECT_SOURCE_DIR}
        ${GLIB_INCLUDE_DIRS}
        ${GSTREAMER_INCLUDE_DIRS}
        ${GSTREAMER_WEBRTC_INCLUDE_DIRS}
        ${GSTREAMER_SDP_INCLUDE_DIRS}
        ${SOUP_INCLUDE_DIRS})

target_link_libraries(${PROJECT_NAME}
        ${GLIB_LIBRARIES}
        ${GSTREAMER_LDFLAGS}
        ${GSTREAMER_WEBRTC_LDFLAGS}
        ${GSTREAMER_SDP_LDFLAGS}
        ${SOUP_LDFLAGS})

install(TARGETS whep-play DESTINATION bin)
