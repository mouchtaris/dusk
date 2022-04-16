cmake_minimum_required(VERSION 3.23.1)

project(string VERSION 1.0)

add_subdirectory(mod/santa.sonar)

add_executable(string src/main.cxx)

target_link_libraries(string PUBLIC
	santa.sonar
)
target_include_directories(string PUBLIC
	"${PROJECT_SOURCE_DIR}/mod/santa.sonar/include"
)
target_compile_options(string PUBLIC
	-std=c++2b -Wall -Wextra
)

set(CMAKE_CXX_STANDARD 23)
set(CMAKE_CXX_STANDARD_REQUIRED True)
