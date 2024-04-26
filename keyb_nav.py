#!/usr/bin/python
# Keyboard navigation
from pytherapy import PushApi, Event, MouseButton, Notifier

api = PushApi()
notify = Notifier()

def pan_view(ev):
    match ev.key:
        case "Left":
            api.pan(0.01, 0.)
        case "Right":
            api.pan(-0.01, 0.)
        case "Up":
            api.pan(0., -0.01)
        case "Down":
            api.pan(0., 0.01)
        case "Z":
            api.zoom(0.95)
        case "X":
            api.zoom(1.05)

def wheel_zoom(ev):
    api.zoom(1 + ev.y/10)
    print(f"Mouse wheel: ({ev.x}, {ev.y})")

for ev in notify:
    match ev.type:
        case Event.KEY_DOWN:
            pan_view(ev)
        case Event.MOUSE_WHEEL:
            wheel_zoom(ev)

