add_library(clauser "")
configure_mzk_component(clauser)

mzk_glob_files(
  CLAUSER_SOURCES
  ${CMAKE_CURRENT_LIST_DIR}/src/**/**/**/*.cpp
  ${CMAKE_CURRENT_LIST_DIR}/src/**/**/*.cpp
  ${CMAKE_CURRENT_LIST_DIR}/src/**/*.cpp
  ${CMAKE_CURRENT_LIST_DIR}/src/*.cpp
  ${CMAKE_CURRENT_LIST_DIR}/src/resources.rc)

mzk_glob_files(
  CLAUSER_HEADERS 
  ${CMAKE_CURRENT_LIST_DIR}/src/**/**/**/*.h
  ${CMAKE_CURRENT_LIST_DIR}/src/**/**/*.h 
  ${CMAKE_CURRENT_LIST_DIR}/src/**/*.h
  ${CMAKE_CURRENT_LIST_DIR}/src/*.h)

target_sources(
  clauser
  PRIVATE ${CLAUSER_SOURCES}
  PUBLIC ${CLAUSER_HEADERS})
  
find_path(REFL_CPP_INCLUDE_DIRS "refl.hpp")

target_include_directories(clauser PUBLIC ${CMAKE_CURRENT_LIST_DIR}/src PRIVATE ${REFL_CPP_INCLUDE_DIRS})

find_package(spdlog CONFIG REQUIRED)
find_package(fmt CONFIG REQUIRED)
find_package(xxHash CONFIG REQUIRED)

target_link_libraries(
  clauser
  PRIVATE 
	spdlog::spdlog
	fmt::fmt
	xxHash::xxhash)
