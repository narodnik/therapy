import zmq
from collections import namedtuple
from . import serial

class Event:
    KEY_DOWN = 0
    MOUSE_MOTION = 1
    MOUSE_WHEEL = 2
    MOUSE_BUTTON_DOWN = 3
    MOUSE_BUTTON_UP = 4

class MouseButton:
    LEFT = 0
    MIDDLE = 1
    RIGHT = 2
    UNKNOWN = 3

EventKeyDown = namedtuple("EventKeyDown", ["type", "key", "mods", "repeat"])
EventMouseMotion = namedtuple("EventMouseMotion", ["type", "x", "y"])
EventMouseWheel = namedtuple("EventKeyDown", ["type", "x", "y"])
EventMouseButtonDown = namedtuple("EventMouseButtonDown", ["type", "button", "x", "y"])
EventMouseButtonUp = namedtuple("EventMouseButtonUp", ["type", "button", "x", "y"])

class Notifier:
    def __init__(self, port=9465):
        context = zmq.Context()
        self.socket = context.socket(zmq.SUB)
        self.socket.connect(f"tcp://localhost:{port}")
        self.socket.setsockopt(zmq.SUBSCRIBE, b'')

    def set_filter(self, ev):
        filter = ev.to_bytes(1, byteorder="little")
        self.socket.setsockopt(zmq.SUBSCRIBE, filter)

    def __iter__(self):
        return self

    def __next__(self):
        event = self.socket.recv()
        cursor = serial.Cursor(event)
        ev_type = serial.read_u8(cursor)
        match ev_type:
            case Event.KEY_DOWN:
                key = serial.decode_str(cursor)
                mods_len = serial.decode_varint(cursor)
                mods = []
                for _ in range(mods_len):
                    mods.append(serial.decode_str(cursor))
                repeat = serial.read_u8(cursor)
                return EventKeyDown(ev_type, key, mods, repeat)
            case Event.MOUSE_MOTION:
                x = serial.read_f32(cursor)
                y = serial.read_f32(cursor)
                return EventMouseMotion(ev_type, x, y)
            case Event.MOUSE_WHEEL:
                x = serial.read_f32(cursor)
                y = serial.read_f32(cursor)
                return EventMouseWheel(ev_type, x, y)
            case Event.MOUSE_BUTTON_DOWN:
                button = serial.read_u8(cursor)
                x = serial.read_f32(cursor)
                y = serial.read_f32(cursor)
                return EventMouseButtonDown(ev_type, button, x, y)
            case Event.MOUSE_BUTTON_UP:
                button = serial.read_u8(cursor)
                x = serial.read_f32(cursor)
                y = serial.read_f32(cursor)
                return EventMouseButtonUp(ev_type, button, x, y)

