#!/usr/bin/env python3
from argparse import ArgumentParser
from enum import IntEnum, auto
from threading import Thread, Lock

import pygame
import zmq

import python_libinput

FPS = 30
RUNNING = True
PATIENTS = {}

CLR_BLACK = (0, 0, 0)
CLR_WHITE = (255, 255, 255)
CLR_RED = (255, 0, 0)
CLR_GREEN = (0, 255, 0)
CLR_CYAN = (0, 255, 255)
CLR_YELLOW = (255, 255, 0)

ZOOM_INC = 0.1

keydown_up = False
keydown_down = False
keydown_right = False
keydown_left = False

buffer_lock = Lock()
buffer = None

class Patient:
    def __init__(self):
        self.down = False
        self.mouse_track = []
        self.wacom_x = 0
        self.wacom_y = 0
        self.view_x = 0
        self.view_y = 0
        self.scale = 1
        self.brush_size = 1
        self.brush_color = CLR_RED

class Event(IntEnum):
    CLOSE = 1
    MOUSE_MOTION = auto()
    MOUSE_DOWN = auto()
    MOUSE_UP = auto()
    ORIGIN = auto()
    SET_COLOR = auto()
    SET_SIZE = auto()
    UNDO = auto()

class ZmqEvent:
    def __init__(self, topic, name):
        self.topic = topic
        self.name = name

    def mouse_motion(self, x, y):
        return f"{self.topic}:{self.name}:{Event.MOUSE_MOTION.value}:{x}:{y}"

    def mouse_down(self):
        return f"{self.topic}:{self.name}:{Event.MOUSE_DOWN.value}"

    def mouse_up(self):
        return f"{self.topic}:{self.name}:{Event.MOUSE_UP.value}"

    def origin(self, x, y):
        return f"{self.topic}:{self.name}:{Event.ORIGIN.value}:{x}:{y}"

    def set_color(self, color):
        return f"{self.topic}:{self.name}:{Event.SET_COLOR.value}:{color[0]}:{color[1]}:{color[2]}"

    def set_size(self, size):
        return f"{self.topic}:{self.name}:{Event.SET_SIZE.value}:{size}"

    def close(self):
        return f"{self.topic}:{self.name}:{Event.CLOSE.value}"

    def undo(self):
        return f"{self.topic}:{self.name}:{Event.UNDO.value}"


def handle_pygame_events(name, pub, ze, is_libbinput_enabled):
    global RUNNING
    global keydown_up, keydown_down, keydown_right, keydown_left

    print("Listening to pygame events...")
    while RUNNING:
        event = pygame.event.wait()

        if event.type == pygame.QUIT:
            print("Received quit!")
            RUNNING = False
            break

        if event.type == pygame.MOUSEMOTION:
            if is_libbinput_enabled:
                continue

            w_x, w_y = event.pos
            # Transform from local (view) to global space
            w_x /= PATIENTS[name].scale
            w_y /= PATIENTS[name].scale
            w_x += PATIENTS[name].view_x
            w_y += PATIENTS[name].view_y
            PATIENTS[name].wacom_x = w_x
            PATIENTS[name].wacom_y = w_y
            if PATIENTS[name].down:
                PATIENTS[name].mouse_track[-1][1].append((w_x, w_y))
                draw_last_segment(name, name)
            pub.send_string(ze.mouse_motion(w_x, w_y))

        elif event.type == pygame.MOUSEBUTTONDOWN:
            if is_libbinput_enabled:
                continue

            PATIENTS[name].down = True
            brush = (PATIENTS[name].brush_size, PATIENTS[name].brush_color)
            PATIENTS[name].mouse_track.append((brush, []))
            pub.send_string(ze.mouse_down())

        elif event.type == pygame.MOUSEBUTTONUP:
            if is_libbinput_enabled:
                continue

            PATIENTS[name].down = False
            pub.send_string(ze.mouse_up())

        elif event.type == pygame.KEYDOWN:
            if event.key == pygame.K_UP:
                keydown_up = True
            elif event.key == pygame.K_DOWN:
                keydown_down = True
            elif event.key == pygame.K_RIGHT:
                keydown_right = True
            elif event.key == pygame.K_LEFT:
                keydown_left = True

            elif event.key == pygame.K_EQUALS:
                PATIENTS[name].scale *= 1 + ZOOM_INC
                with buffer_lock:
                    rect = buffer.get_rect()
                    screen_w, screen_h = rect.w, rect.h
                # we want to center zoom so also offset the origin
                PATIENTS[name].view_x += ZOOM_INC*screen_w/2
                PATIENTS[name].view_y += ZOOM_INC*screen_h/2
                redraw(name)
            elif event.key == pygame.K_MINUS:
                PATIENTS[name].scale *= 1 - ZOOM_INC
                with buffer_lock:
                    rect = buffer.get_rect()
                    screen_w, screen_h = rect.w, rect.h
                PATIENTS[name].view_x -= ZOOM_INC*screen_w/2
                PATIENTS[name].view_y -= ZOOM_INC*screen_h/2
                redraw(name)

            elif event.key == pygame.K_0:
                PATIENTS[name].brush_color = CLR_BLACK
                pub.send_string(ze.set_color(CLR_BLACK))
            elif event.key == pygame.K_1:
                PATIENTS[name].brush_color = CLR_WHITE
                pub.send_string(ze.set_color(CLR_WHITE))
            elif event.key == pygame.K_2:
                PATIENTS[name].brush_color = CLR_RED
                pub.send_string(ze.set_color(CLR_RED))
            elif event.key == pygame.K_3:
                PATIENTS[name].brush_color = CLR_GREEN
                pub.send_string(ze.set_color(CLR_GREEN))
            elif event.key == pygame.K_4:
                PATIENTS[name].brush_color = CLR_CYAN
                pub.send_string(ze.set_color(CLR_CYAN))
            elif event.key == pygame.K_5:
                PATIENTS[name].brush_color = CLR_YELLOW
                pub.send_string(ze.set_color(CLR_YELLOW))

            elif event.key == pygame.K_q:
                PATIENTS[name].brush_size = 1
                pub.send_string(ze.set_size(1))
            elif event.key == pygame.K_w:
                PATIENTS[name].brush_size = 2
                pub.send_string(ze.set_size(2))
            elif event.key == pygame.K_e:
                PATIENTS[name].brush_size = 3
                pub.send_string(ze.set_size(3))
            elif event.key == pygame.K_r:
                PATIENTS[name].brush_size = 4
                pub.send_string(ze.set_size(4))
            elif event.key == pygame.K_t:
                PATIENTS[name].brush_size = 5
                pub.send_string(ze.set_size(5))
            elif event.key == pygame.K_y:
                PATIENTS[name].brush_size = 6
                pub.send_string(ze.set_size(6))

            elif event.key == pygame.K_z:
                trk = PATIENTS[name].mouse_track
                try:
                    idx = len(trk) - 1 - next(i for i, tpl in enumerate(trk[::-1]) if tpl[1] == [])
                    pub.send_string(ze.undo())
                except StopIteration:
                    idx = len(trk) - 1
                PATIENTS[name].mouse_track = trk[:idx]


        elif event.type == pygame.KEYUP:
            if event.key == pygame.K_UP:
                keydown_up = False
            elif event.key == pygame.K_DOWN:
                keydown_down = False
            elif event.key == pygame.K_RIGHT:
                keydown_right = False
            elif event.key == pygame.K_LEFT:
                keydown_left = False

        """ Origin is a bit buggy
        if keydown_up:
            PATIENTS[name].view_y -= 20
        if keydown_down:
            PATIENTS[name].view_y += 20
        if keydown_left:
            PATIENTS[name].view_x -= 20
        if keydown_right:
            PATIENTS[name].view_x += 20

        if keydown_up or keydown_down or keydown_left or keydown_right:
            msg = ze.origin(PATIENTS[name].view_x, PATIENTS[name].view_y)
            pub.send_string(msg)
        """

    pub.send_string(ze.close())
    pub.close()


def handle_libinput_events(name, pub, ze, screen):
    li = python_libinput.libinput()
    assert li.start()

    print("Listening to libinput events...")
    while RUNNING and li.wait() > -1:
        events = li.poll()
        for event in events:
            # tip up / down
            if event.type == 0:
                if event.tip_is_down:
                    PATIENTS[name].down = True
                    brush = (PATIENTS[name].brush_size, PATIENTS[name].brush_color)
                    PATIENTS[name].mouse_track.append((brush, []))
                    pub.send_string(ze.mouse_down())
                else:
                    PATIENTS[name].down = False
                    pub.send_string(ze.mouse_up())
            # cursor move
            elif event.type == 1:
                x, y = event.x, event.y
                size = screen.get_rect()
                w, h = size.w, size.h
                w_x, w_y = x * w, y * h
                PATIENTS[name].wacom_x = w_x
                PATIENTS[name].wacom_y = w_y
                if PATIENTS[name].down:
                    PATIENTS[name].mouse_track[-1][1].append((w_x, w_y))
                pub.send_string(ze.mouse_motion(w_x, w_y))

    pub.close()


def handle_zmq_events(name, sub):
    print("Listening to ZMQ events...")
    while RUNNING:
        msg = sub.recv_string()  # This hangs once when RUNNING=False
        msg = msg.split(":")

        _topic = msg[0]
        patient = msg[1]
        event = Event(int(msg[2]))

        # Skip ourselves
        if name == patient:
            continue

        if not PATIENTS.get(patient):
            PATIENTS[patient] = Patient()

        if event == Event.MOUSE_MOTION:
            w_x, w_y = float(msg[3]), float(msg[4])
            PATIENTS[patient].wacom_x = w_x
            PATIENTS[patient].wacom_y = w_y
            if PATIENTS[patient].down:
                PATIENTS[patient].mouse_track[-1][1].append((w_x, w_y))
                draw_last_segment(name, patient)

        elif event == Event.MOUSE_DOWN:
            PATIENTS[patient].down = True
            brush = (PATIENTS[patient].brush_size, PATIENTS[patient].brush_color)
            PATIENTS[patient].mouse_track.append((brush, []))

        elif event == Event.MOUSE_UP:
            PATIENTS[patient].down = False

        elif event == Event.ORIGIN:
            o_x, o_y = int(msg[3]), int(msg[4])
            PATIENTS[patient].view_x = o_x
            PATIENTS[patient].view_y = o_y

        elif event == Event.SET_COLOR:
            r, g, b = int(msg[3]), int(msg[4]), int(msg[5])
            PATIENTS[patient].brush_color = (r, g, b)

        elif event == Event.SET_SIZE:
            PATIENTS[patient].brush_size = int(msg[3])

        elif event == Event.UNDO:
            trk = PATIENTS[patient].mouse_track
            try:
                idx = len(trk) - 1 - next(i for i, tpl in enumerate(trk[::-1]) if tpl[1] == [])
            except StopIteration:
                idx = len(trk) - 1
            PATIENTS[patient].mouse_track = trk[:idx]

            # redraw everything
            redraw(name)

    sub.close()

def redraw(name):
    p = PATIENTS[name]
    with buffer_lock:
        assert buffer is not None
        buffer.fill(pygame.Color("black"))

        for patient in PATIENTS.values():
            for brush, segment in patient.mouse_track:
                if not segment:
                    continue

                start = segment[0]
                size, clr = brush
                start_x, start_y = start
                start_x -= p.view_x
                start_y -= p.view_y

                start_x *= p.scale
                start_y *= p.scale

                for end in segment[1:]:
                    end_x, end_y = end
                    end_x -= p.view_x
                    end_y -= p.view_y

                    end_x *= p.scale
                    end_y *= p.scale

                    pygame.draw.line(buffer, clr,
                                     (start_x, start_y),
                                     (end_x, end_y),
                                     width=size)

                    start_x, start_y = end_x, end_y

def draw_last_segment(name, patient):
    p = PATIENTS[name]
    with buffer_lock:
        assert buffer is not None

        brush, segment = PATIENTS[patient].mouse_track[-1]
        if len(segment) < 2:
            return

        start, end = segment[-2], segment[-1]
        size, clr = brush
        start_x, start_y = start
        start_x -= p.view_x
        start_y -= p.view_y
        end_x, end_y = end
        end_x -= p.view_x
        end_y -= p.view_y

        start_x *= p.scale
        start_y *= p.scale
        end_x *= p.scale
        end_y *= p.scale

        pygame.draw.line(buffer, clr,
                         (start_x, start_y),
                         (end_x, end_y),
                         width=size)

def main(frontend, backend, name, topic, is_libinput_enabled):
    global buffer

    pygame.init()
    pygame.display.set_caption("Therapy Session")

    flags = pygame.RESIZABLE | pygame.DOUBLEBUF
    screen = pygame.display.set_mode((800, 600), flags)
    buffer = screen.copy()

    p = Patient()
    PATIENTS[name] = p

    ctx = zmq.Context()
    ze = ZmqEvent(topic, name)

    # Input event loops

    pygame_pub = ctx.socket(zmq.PUB)
    pygame_pub.connect(backend)

    pygame_event_thread = Thread(target=handle_pygame_events,
                                 args=(name, pygame_pub, ze,
                                       is_libinput_enabled))
    pygame_event_thread.start()

    if is_libinput_enabled:
        libinput_pub = ctx.socket(zmq.PUB)
        libinput_pub.connect(backend)

        libinput_event_thread = Thread(target=handle_libinput_events,
                                       args=(name, libinput_pub, ze,
                                             screen))
        libinput_event_thread.start()

    # Network event loop
    sub = ctx.socket(zmq.SUB)
    sub.connect(frontend)
    sub.setsockopt_string(zmq.SUBSCRIBE, topic)
    zmq_event_thread = Thread(target=handle_zmq_events, args=(name, sub))
    zmq_event_thread.start()

    print("Starting game loop...")
    clock = pygame.time.Clock()
    while RUNNING:
        # We do these here because the input event loop is blocking
        if keydown_up:
            p.view_y -= 10
            redraw(name)
        elif keydown_down:
            p.view_y += 10
            redraw(name)

        if keydown_left:
            p.view_x -= 10
            redraw(name)
        elif keydown_right:
            p.view_x += 10
            redraw(name)

        with buffer_lock:
            screen.blit(buffer, (0, 0))

        for patient in PATIENTS.values():
            # Transform from global to local (view) space
            wacom_x = patient.wacom_x - p.view_x
            wacom_y = patient.wacom_y - p.view_y
            wacom_x *= p.scale
            wacom_y *= p.scale
            pygame.draw.circle(screen,
                               patient.brush_color,
                               (wacom_x, wacom_y),
                               patient.brush_size * 2)

        pygame.display.flip()
        clock.tick(FPS)

    zmq_event_thread.join()
    pygame_event_thread.join()
    pygame.quit()


if __name__ == "__main__":
    parser = ArgumentParser()
    parser.add_argument("-f", "--frontend", default="tcp://127.0.0.1:5559")
    parser.add_argument("-b", "--backend", default="tcp://127.0.0.1:5560")
    parser.add_argument("-p", "--patient", required=True, type=str)
    parser.add_argument("-i", "--libinput", action="store_true")
    parser.add_argument("-t", "--topic", default="T")
    args = parser.parse_args()

    try:
        main(args.frontend, args.backend, args.patient, args.topic,
             args.libinput)
    except KeyboardInterrupt:
        print("\rCaught interrupt. Stopping...")
        RUNNING = False
