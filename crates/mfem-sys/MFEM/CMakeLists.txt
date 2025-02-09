cmake_minimum_required (VERSION 3.12 FATAL_ERROR)
project (MfemPackageConfig)

set(CMAKE_FIND_ROOT_PATH_MODE_PACKAGE "BOTH" CACHE PATH "")

message(STATUS "Looking for mfem ...")
find_package(MFEM REQUIRED CONFIG)

if (NOT DEFINED MFEM_INCLUDE_DIRS)
  set(MFEM_INCLUDE_DIRS "")
endif()

file (WRITE ${CMAKE_BINARY_DIR}/mfem_info.txt
      "VERSION=${MFEM_VERSION}\n"
      "MFEM_LIBRARIES=${MFEM_LIBRARIES}\n"
      "INCLUDE_DIRS=${MFEM_INCLUDE_DIRS}\n"
      "LIBRARY_DIR=${MFEM_LIBRARY_DIR}\n"
      "CXX_FLAGS=${MFEM_CXX_FLAGS}\n"
    )

install (FILES ${CMAKE_BINARY_DIR}/mfem_info.txt TYPE DATA)
