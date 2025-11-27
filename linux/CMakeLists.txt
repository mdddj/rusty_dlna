cmake_minimum_required(VERSION 3.10)
set(PROJECT_NAME "rusty_dlna")
project(${PROJECT_NAME} LANGUAGES CXX)
include("../cargokit/cmake/cargokit.cmake")
apply_cargokit(${PROJECT_NAME} ../rust rusty_dlna "")
set(rusty_dlna_bundled_libraries
  "${${PROJECT_NAME}_cargokit_lib}"
  PARENT_SCOPE
)