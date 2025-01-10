include(FetchContent)

set(ROOT_DIR "${CMAKE_CURRENT_LIST_DIR}/..")
set(QT_BUILD_VERSION "6.2.2")

if(WIN32)
  set(QT_BUILD_COMPILER "msvc2019_64")
elseif(LINUX)
  set(QT_BUILD_COMPILER "gcc_64")
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
  # GIT_REPOSITORY "https://github.com/horsicq/die_library"
  # GIT_TAG ebe34ba3b3a38d5f40c02064a116faec7376bad3
  GIT_REPOSITORY "https://github.com/calladoum-elastic/die_library"
  GIT_TAG e6f91085fed31b69907ab03f95d6bf8dce7752ac
)

set(DIE_BUILD_AS_STATIC ON CACHE INTERNAL "")
FetchContent_MakeAvailable( DieLibrary )

message(STATUS "Using DieLibrary in '${dielibrary_SOURCE_DIR}'")

list(APPEND CMAKE_MODULE_PATH
  "${dielibrary_SOURCE_DIR}/dep/build_tools/cmake"
)
