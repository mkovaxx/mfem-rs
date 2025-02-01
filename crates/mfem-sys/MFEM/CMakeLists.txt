cmake_minimum_required (VERSION 3.10 FATAL_ERROR)
project (MfemPackageConfig)

set(CMAKE_FIND_ROOT_PATH_MODE_PACKAGE "BOTH" CACHE PATH "")

message(STATUS "Looking for mfem ...")
set(MFEM_DIR "" CACHE PATH "Path to the MFEM build or install prefix.")
if (MFEM_DIR)
   find_package(mfem REQUIRED NAMES MFEM HINTS "${MFEM_DIR}"
                "${MFEM_DIR}/lib/cmake/mfem" NO_DEFAULT_PATH)
else()
   find_package(mfem REQUIRED NAMES MFEM)
endif()

file (WRITE ${CMAKE_BINARY_DIR}/mfem_info.txt
      "VERSION=${MFEM_VERSION}\n"
      "MFEM_LIBRARIES=${MFEM_LIBRARIES}\n"
      "INCLUDE_DIRS=${MFEM_INCLUDE_DIRS}\n"
      "LIBRARY_DIR=${MFEM_LIBRARY_DIR}\n"
      "CXX_FLAGS=${MFEM_CXX_FLAGS}\n"
)

install (FILES ${CMAKE_BINARY_DIR}/mfem_info.txt TYPE DATA)
