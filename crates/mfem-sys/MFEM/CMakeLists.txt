cmake_minimum_required (VERSION 3.1 FATAL_ERROR)
project (MfemPackageConfig)

set(CMAKE_FIND_ROOT_PATH_MODE_PACKAGE "BOTH" CACHE PATH "")

find_package (MFEM REQUIRED)

file (WRITE ${CMAKE_BINARY_DIR}/mfem_info.txt
      "VERSION=${MFEM_VERSION}\n"
      "INCLUDE_DIR=${MFEM_INCLUDE_DIR}\n"
      "LIBRARY_DIR=${MFEM_LIBRARY_DIR}\n")

install (FILES ${CMAKE_BINARY_DIR}/mfem_info.txt TYPE DATA)
