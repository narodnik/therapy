#include <stdio.h>
#include <fcntl.h>
#include <errno.h>
#include <poll.h>
#include <string.h>
#include <unistd.h>
#include <libinput.h>
#include <pybind11/pybind11.h>
#include <pybind11/stl.h>

static int open_restricted(const char *path, int flags, void *user_data)
{
	int fd = open(path, flags);

	if (fd < 0) {
		fprintf(stderr, "Failed to open %s (%s)\n",
			path, strerror(errno));
        return -errno;
    }

	return fd;
}

static void close_restricted(int fd, void *user_data)
{
	close(fd);
}

static const struct libinput_interface interface = {
	.open_restricted = open_restricted,
	.close_restricted = close_restricted,
};

struct event {
    int type;
    bool tip_is_down;
    double x, y;
};

class libinput {
public:
    libinput();
    ~libinput();

    bool start();
    std::vector<event> do_poll();
    int wait();

private:
    //libinput_device *device;
    udev *ud = NULL;
    libinput *li = NULL;
    pollfd fd;
};

libinput::libinput() {
    fd.fd = -1;
    fd.events = POLLIN;
    fd.revents = 0;
}
libinput::~libinput() {
    libinput_unref(li);
    udev_unref(ud);
    close(fd.fd);
    fd.fd = -1;
}

bool libinput::start() {
    ud = udev_new();

    li = libinput_udev_create_context(&interface, NULL, ud);
	if (!li) {
		fprintf(stderr, "Failed to initialize path context\n");
        udev_unref(ud);
		return false;
	}

	//if (verbose) {
	//	libinput_log_set_handler(li, log_handler);
	//	libinput_log_set_priority(li, LIBINPUT_LOG_PRIORITY_DEBUG);
	//}

    if (libinput_udev_assign_seat(li, "seat0")) {
		fprintf(stderr, "Failed to set seat\n");
        libinput_unref(li);
        udev_unref(ud);
        return false;
    }

    /*
    const char *device_path = "/dev/input/event24";
    device = libinput_path_add_device(li, device_path);
    if (!device) {
        fprintf(stderr, "Failed to initialize device %s\n", device_path);
        libinput_unref(li);
        li = NULL;
        return false;
    }
    */

    fd.fd = libinput_get_fd(li);

    return true;
}

std::vector<event> libinput::do_poll() {
	struct libinput_event *ev;
    struct libinput_event_tablet_tool *t = NULL;

    std::vector<event> events;

    libinput_dispatch(li);
    while ((ev = libinput_get_event(li))) {
        switch (libinput_event_get_type(ev)) {
            case LIBINPUT_EVENT_NONE:
                abort();
            case LIBINPUT_EVENT_DEVICE_ADDED:
                // No need to do anything
                break;
            case LIBINPUT_EVENT_DEVICE_REMOVED:
                // No need to do anything
                break;
            case LIBINPUT_EVENT_TABLET_TOOL_TIP: {
                t = libinput_event_get_tablet_tool_event(ev);

                bool tip_is_down = (
                    libinput_event_tablet_tool_get_tip_state(t)
                    == LIBINPUT_TABLET_TOOL_TIP_DOWN
                );

                events.push_back(event { 0, tip_is_down, 0, 0 });

                /*if (tip_is_down)
                    puts("tip");
                else
                    puts("tip up");*/
                break;
            }
            case LIBINPUT_EVENT_TABLET_TOOL_AXIS: {
                t = libinput_event_get_tablet_tool_event(ev);

                //double x = libinput_event_tablet_tool_get_x(t);
                //double y = libinput_event_tablet_tool_get_y(t);
                double x = libinput_event_tablet_tool_get_x_transformed(t, 1.0);
                double y = libinput_event_tablet_tool_get_y_transformed(t, 1.0);

                //printf("%.4f %.4f\n", x, y);
                events.push_back(event { 1, false, x, y });

                break;
            }

            default:
                break;
        }

        libinput_event_destroy(ev);
    }
    return events;
}

int libinput::wait() {
    return poll(&fd, 1, -1);
}

namespace py = pybind11;
PYBIND11_MODULE(python_libinput, m) {
    py::class_<libinput>(m, "libinput")
        .def(py::init<>())
        .def("start", &libinput::start)
        .def("poll", &libinput::do_poll)
        .def("wait", &libinput::wait)
    ;
    py::class_<event>(m, "event")
        .def_readonly("type", &event::type)
        .def_readonly("tip_is_down", &event::tip_is_down)
        .def_readonly("x", &event::x)
        .def_readonly("y", &event::y)
    ;
}

