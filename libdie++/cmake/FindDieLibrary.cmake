include(FetchContent)

set(ROOT_DIR "${CMAKE_CURRENT_LIST_DIR}/..")
set(QT_BUILD_VERSION "6.10.0")

if(WIN32)
  set(QT_BUILD_COMPILER "msvc2022_64")
elseif(LINUX)
  if(CMAKE_SYSTEM_PROCESSOR MATCHES "arm64|aarch64")
    set(QT_BUILD_COMPILER "gcc_arm64")
  else()
    set(QT_BUILD_COMPILER "gcc_64")
  endif()
elseif(APPLE)
  set(QT_BUILD_COMPILER "macos")
else()
  message(FATAL "nope")
endif()

set(Qt6_PREFIX_PATH "${ROOT_DIR}/build/${QT_BUILD_VERSION}/${QT_BUILD_COMPILER}")
set(CMAKE_PREFIX_PATH  "${Qt6_PREFIX_PATH}")
set(Qt6_CMAKE_PREFIX_PATH "${Qt6_PREFIX_PATH}/lib/cmake")
set(Qt6_DIR ${Qt6_CMAKE_PREFIX_PATH}/Qt6)
set(QT_DIR ${Qt6_DIR})

list(APPEND CMAKE_MODULE_PATH
  ${Qt6_CMAKE_PREFIX_PATH}
  ${Qt6_DIR}
)

find_package(Qt6 REQUIRED COMPONENTS Core Concurrent Qml)

FetchContent_Declare(
  DieLibrary
  GIT_REPOSITORY "https://github.com/horsicq/die_library"
  GIT_TAG 06e36e4136018c294d72cc9450521a6dbb2bf81e
)

set(DIE_BUILD_AS_STATIC ON CACHE INTERNAL "")
FetchContent_MakeAvailable( DieLibrary )

message(STATUS "Using DieLibrary in '${dielibrary_SOURCE_DIR}'")

list(APPEND CMAKE_MODULE_PATH
  "${dielibrary_SOURCE_DIR}/dep/build_tools/cmake"
)
