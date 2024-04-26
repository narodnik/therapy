#!/usr/bin/python
# Mouse pencil tool
from pytherapy import PushApi, Event, MouseButton, Notifier

MOUSE_STATE = 0
current_pos = (0., 0.)

PEERS = [
    #"[XXXX:XXXX:XXXX:XXXX:XXXX:XXXX:XXXX:XXXX]",
]

api = PushApi()
peers_api = [PushApi(peer) for peer in PEERS]
notify = Notifier()

def draw_line(layer_name, x1, y1, x2, y2, thickness, r, g, b, a):
    for peer_api in [api] + peers_api:
        peer_api.draw_line(
            layer_name,
            x1, y1, x2, y2, thickness,
            r, g, b, a
        )

for ev in notify:
    match ev.type:
        case Event.MOUSE_BUTTON_DOWN:
            if ev.button == MouseButton.LEFT:
                MOUSE_STATE = 1
                current_pos = (ev.x, ev.y)
            print(f"Mouse button down: {ev.button} @ ({ev.x}, {ev.y})")
        case Event.MOUSE_BUTTON_UP:
            if ev.button == MouseButton.LEFT:
                MOUSE_STATE = 0
                current_pos = (ev.x, ev.y)
            print(f"Mouse button up: {ev.button} @ ({ev.x}, {ev.y})")
        case Event.MOUSE_MOTION:
            if MOUSE_STATE == 1:
                x0, y0 = current_pos
                draw_line(
                    "genjix",
                    x0, y0, ev.x, ev.y, 0.001,
                    1.0, 0.0, 0.0, 1.0
                )
                current_pos = (ev.x, ev.y)

