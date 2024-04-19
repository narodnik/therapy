.POSIX:

PYTHON_VERSION = $(shell python3 -V | cut -d' ' -f2 | cut -d'.' -f1-2)
PYTHON_CFLAGS = $(shell pkg-config --cflags python-$(PYTHON_VERSION))
PYTHON_LIBS = $(shell pkg-config --libs python-$(PYTHON_VERSION))
PYTHON_SO_SUFFIX = $(shell python3-config --extension-suffix)

LIBINPUT_CFLAGS = $(shell pkg-config --cflags libinput)
LIBINPUT_LIBS = $(shell pkg-config --libs libinput)

CC = g++

CFLAGS = -O3 -Wall -std=c++11 -fPIC $(PYTHON_CFLAGS) $(LIBINPUT_CFLAGS) -I./
LDFLAGS = -shared $(PYTHON_LIBS) $(LIBINPUT_LIBS)

HDR = \
	pybind11/attr.h \
	pybind11/buffer_info.h \
	pybind11/cast.h \
	pybind11/chrono.h \
	pybind11/common.h \
	pybind11/complex.h \
	pybind11/detail \
	pybind11/detail/class.h \
	pybind11/detail/common.h \
	pybind11/detail/descr.h \
	pybind11/detail/init.h \
	pybind11/detail/internals.h \
	pybind11/detail/type_caster_base.h \
	pybind11/detail/typeid.h \
	pybind11/eigen \
	pybind11/eigen.h \
	pybind11/eigen/matrix.h \
	pybind11/eigen/tensor.h \
	pybind11/embed.h \
	pybind11/eval.h \
	pybind11/functional.h \
	pybind11/gil.h \
	pybind11/iostream.h \
	pybind11/numpy.h \
	pybind11/operators.h \
	pybind11/options.h \
	pybind11/pybind11.h \
	pybind11/pytypes.h \
	pybind11/stl \
	pybind11/stl_bind.h \
	pybind11/stl/filesystem.h \
	pybind11/stl.h

LIB = ../python_libinput$(PYTHON_SO_SUFFIX)

all: $(LIB)

$(LIB): $(HDR) python-libinput.cpp
	$(CC) $(CFLAGS) python-libinput.cpp $(LDFLAGS) -o $@

.PHONY: all
