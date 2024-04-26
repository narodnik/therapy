#!/usr/bin/python
# Mouse pencil tool
import python_libinput
from pytherapy import PushApi, ReqApi, Event, MouseButton, Notifier

CURSOR_SIZE = 0.02
CURSOR_COLOR = (1.0, 0.0, 0.0, 1.0)
LINE_COLOR = (1.0, 0.0, 0.0, 1.0)

PEERS = [
    #"[XXXX:XXXX:XXXX:XXXX:XXXX:XXXX:XXXX:XXXX]",
]

MOUSE_STATE = 0
current_pos = None

api = PushApi()
reqapi = ReqApi()
peers_api = [PushApi(peer) for peer in PEERS]

def draw_line(layer_name, x1, y1, x2, y2, thickness, r, g, b, a):
    for peer_api in [api] + peers_api:
        peer_api.draw_line(
            layer_name,
            x1, y1, x2, y2, thickness,
            r, g, b, a
        )

li = python_libinput.libinput()
assert li.start()

reqapi.delete_layer("wacom_cursor")
# Draw a crosshair
api.draw_line(
    "wacom_cursor",
    -CURSOR_SIZE, 0.0, CURSOR_SIZE, 0.0, 0.001,
    *CURSOR_COLOR
)
api.draw_line(
    "wacom_cursor",
    0.0, -CURSOR_SIZE, 0.0, CURSOR_SIZE, 0.001,
    *CURSOR_COLOR
)

while True:
    events = li.poll()
    for event in events:
        # tip up / down
        if event.type == 0:
            if event.tip_is_down:
                MOUSE_STATE = 1
                reqapi.hide_layer("wacom_cursor")
            else:
                MOUSE_STATE = 0
                current_pos = None
                reqapi.show_layer("wacom_cursor")
        # cursor move
        elif event.type == 1:
            x, y = event.x, event.y
            x, y = reqapi.screen_to_world(x, y)
            #print(x, y)
            if MOUSE_STATE == 1:
                if current_pos is None:
                    current_pos = (x, y)
                    continue

                x0, y0 = current_pos
                draw_line(
                    "genjix",
                    x0, y0, x, y, 0.001,
                    *LINE_COLOR
                )
                current_pos = (x, y)
            else:
                reqapi.set_layer_pos("wacom_cursor", x, y)

