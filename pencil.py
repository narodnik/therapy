#!/usr/bin/python
# Mouse pencil tool
from pytherapy import Api, Event, MouseButton, Notifier

MOUSE_STATE = 0
current_pos = (0., 0.)

api = Api()
notify = Notifier()
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
                api.draw_line(
                    "genjix",
                    x0, y0, ev.x, ev.y, 0.001,
                    1.0, 0.0, 0.0, 1.0
                )
                current_pos = (ev.x, ev.y)

