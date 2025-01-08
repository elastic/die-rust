cmake_minimum_required(VERSION 3.26)

project(
  DIE
  VERSION 0.1.0
  LANGUAGES CXX
  DESCRIPTION "DIE Library implementation"
)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED True)
set(CMAKE_CXX_EXTENSIONS OFF)
set(VCPKG_MANIFEST_MODE OFF)
set(CXX_STANDARD 20)

set(DIE_ROOT_DIR ${CMAKE_CURRENT_LIST_DIR})

list(APPEND CMAKE_MODULE_PATH "${DIE_ROOT_DIR}/cmake")
set_property(GLOBAL PROPERTY USE_FOLDERS ON)

find_package(DieLibrary REQUIRED)

set(DIE_CXX_ROOT_DIR ${DIE_ROOT_DIR})
set(DIELIB_BASE_ROOT ${dielibrary_SOURCE_DIR})
set(DIE_BASE_ROOT "${dielibrary_SOURCE_DIR}/dep/Detect-It-Easy")

file(STRINGS ${DIELIB_BASE_ROOT}/release_version.txt DIELIB_VERSION)
file(STRINGS ${DIE_BASE_ROOT}/die_version.txt DIE_VERSION)

add_library(die++ STATIC ./src/die.cpp)
add_dependencies(die++ die)

target_include_directories(
    die++
    PRIVATE
        ${DIE_CXX_ROOT_DIR}/inc
        ${DIELIB_BASE_ROOT}/src/include
)

if(MSVC)
    # Disable MSVC `min` & `max` macros for `die` (breaks Qt)
    target_compile_definitions(die PUBLIC NOMINMAX)
endif()

if(APPLE)
    target_link_libraries(die++ PRIVATE dl "-framework CoreFoundation")
    target_link_options(die++ PRIVATE -Wl,-rpath,$ORIGIN/lib)
endif()

if(LINUX)
    target_link_options(die++ PRIVATE -Wl,-rpath,$ORIGIN/lib)
endif()

target_compile_definitions(die++
    PRIVATE
        DIE_VERSION="${DIE_VERSION}"
        DIELIB_VERSION="${DIELIB_VERSION}"
)

target_link_libraries(die++ PRIVATE $<TARGET_FILE:die> $<TARGET_PROPERTY:die,LINK_LIBRARIES>)
target_link_libraries(die++ PRIVATE Qt6::Core)
target_link_libraries(die++ PRIVATE Qt6::Qml)
target_link_libraries(die++ PRIVATE Qt6::Concurrent)
target_link_libraries(die++ PRIVATE Qt6::Network)

install(TARGETS die++ DESTINATION die/)
install(TARGETS die DESTINATION die/)
install(DIRECTORY ${DIELIB_BASE_ROOT}/dep/Detect-It-Easy/db DESTINATION die/db)
install(DIRECTORY ${DIELIB_BASE_ROOT}/dep/Detect-It-Easy/db_custom DESTINATION die/db)

if(LINUX OR APPLE)
    install(
        DIRECTORY
            ${Qt6_DIR}/../../
        DESTINATION
            die/lib
        FILES_MATCHING
            PATTERN "libQt6Core.*"
            PATTERN "libQt6Qml.*"
            PATTERN "libQt6Concurrent.*"
            PATTERN "libQt6Network.*"
            PATTERN "libicui18n.*"
            PATTERN "libicuuc.*"
            PATTERN "libicudata.*"
            PATTERN "cmake" EXCLUDE
            PATTERN "objects-*" EXCLUDE
            PATTERN "pkgconfig" EXCLUDE
    )
else()
    install(
        DIRECTORY
            ${Qt6_DIR}/../../../bin/
        DESTINATION
            die/
        FILES_MATCHING
            PATTERN "Qt6Core.*"
            PATTERN "Qt6Qml.*"
            PATTERN "Qt6Concurrent.*"
            PATTERN "Qt6Network.*"
            PATTERN "Qt6Cored.*"
            PATTERN "Qt6Qmld.*"
            PATTERN "Qt6Concurrentd.*"
            PATTERN "Qt6Networkd.*"
    )
endif()
