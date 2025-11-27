# The Flutter tooling requires that developers have CMake 3.10 or later
# installed. You should not increase this version, as doing so will cause
# the plugin to fail to compile for some customers of the plugin.
cmake_minimum_required(VERSION 3.10)

project(rusty_dlna_library VERSION 0.0.1 LANGUAGES C)

add_library(rusty_dlna SHARED
  "rusty_dlna.c"
)

set_target_properties(rusty_dlna PROPERTIES
  PUBLIC_HEADER rusty_dlna.h
  OUTPUT_NAME "rusty_dlna"
)

target_compile_definitions(rusty_dlna PUBLIC DART_SHARED_LIB)

if (ANDROID)
  # Support Android 15 16k page size
  target_link_options(rusty_dlna PRIVATE "-Wl,-z,max-page-size=16384")
endif()
