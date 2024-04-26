import zmq
from . import serial

COMMAND_HELLO = 0
COMMAND_DRAWLINE = 1
COMMAND_PAN = 2
COMMAND_ZOOM = 3
COMMAND_SCREENTOWORLD = 4
COMMAND_GETLAYERS = 5
COMMAND_DELETELAYER = 6
COMMAND_SHOWLAYER = 7
COMMAND_HIDELAYER = 8
COMMAND_SETLAYERPOS = 9
COMMAND_SCREENSIZE = 10

class PushApi:

    def __init__(self, addr="localhost", port=9466):
        context = zmq.Context()
        self.socket = context.socket(zmq.PUB)
        self.socket.connect(f"tcp://{addr}:{port}")

    def _push_cmd(self, cmd, payload):
        req_cmd = bytearray()
        serial.write_u8(req_cmd, cmd)
        self.socket.send_multipart([req_cmd, payload])

    def draw_line(self, layer_name,
                  x1, y1, x2, y2, thickness,
                  r, g, b, a):
        req = bytearray()
        serial.encode_str(req, layer_name)
        serial.write_f32(req, x1)
        serial.write_f32(req, y1)
        serial.write_f32(req, x2)
        serial.write_f32(req, y2)
        serial.write_f32(req, thickness)
        serial.write_f32(req, r)
        serial.write_f32(req, g)
        serial.write_f32(req, b)
        serial.write_f32(req, a)
        self._push_cmd(COMMAND_DRAWLINE, req)

    def pan(self, x, y):
        req = bytearray()
        serial.write_f32(req, x)
        serial.write_f32(req, y)
        _ = self._push_cmd(COMMAND_PAN, req)

    def zoom(self, scale):
        req = bytearray()
        serial.write_f32(req, scale)
        _ = self._push_cmd(COMMAND_ZOOM, req)

class ReqApi:

    def __init__(self, addr="localhost", port=9464):
        context = zmq.Context()
        self.socket = context.socket(zmq.REQ)
        self.socket.connect(f"tcp://{addr}:{port}")

    def _make_request(self, cmd, payload):
        req_cmd = bytearray()
        serial.write_u8(req_cmd, cmd)
        self.socket.send_multipart([req_cmd, payload])

        reply = self.socket.recv()
        cursor = serial.Cursor(reply)
        return cursor

    def hello(self):
        response = self._make_request(COMMAND_HELLO, bytearray())
        return serial.decode_str(response)

    def screen_to_world(self, x, y):
        req = bytearray()
        serial.write_f32(req, x)
        serial.write_f32(req, y)
        cur = self._make_request(COMMAND_SCREENTOWORLD, req)
        x = serial.read_f32(cur)
        y = serial.read_f32(cur)
        return (x, y)

    def get_layers(self):
        cur = self._make_request(COMMAND_GETLAYERS, bytearray())
        layers_len = serial.decode_varint(cur)
        layers = []
        for _ in range(layers_len):
            layers.append(serial.decode_str(cur))
        return layers

    def delete_layer(self, layer_name):
        req = bytearray()
        serial.encode_str(req, layer_name)
        cur = self._make_request(COMMAND_DELETELAYER, req)
        is_success = serial.read_u8(cur)
        return bool(is_success)

    def show_layer(self, layer_name):
        req = bytearray()
        serial.encode_str(req, layer_name)
        cur = self._make_request(COMMAND_SHOWLAYER, req)
        is_success = serial.read_u8(cur)
        return bool(is_success)

    def hide_layer(self, layer_name):
        req = bytearray()
        serial.encode_str(req, layer_name)
        cur = self._make_request(COMMAND_HIDELAYER, req)
        is_success = serial.read_u8(cur)
        return bool(is_success)

    def set_layer_pos(self, layer_name, x, y):
        req = bytearray()
        serial.encode_str(req, layer_name)
        serial.write_f32(req, x)
        serial.write_f32(req, y)
        cur = self._make_request(COMMAND_SETLAYERPOS, req)
        is_success = serial.read_u8(cur)
        return bool(is_success)

    def screen_size(self):
        cur = self._make_request(COMMAND_SCREENSIZE, bytearray())
        w = serial.read_f32(cur)
        h = serial.read_f32(cur)
        return (w, h)

