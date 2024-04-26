#!/usr/bin/python
# Draw lines by clicking
from pytherapy import Api, Event, MouseButton, Notifier

points = []

api = Api()
notify = Notifier()
notify.set_filter(Event.MOUSE_BUTTON_DOWN)

for ev in notify:
    if ev.button == MouseButton.LEFT:
        points.append((ev.x, ev.y))
        if len(points) == 2:
            p1x, p1y = points[0]
            p2x, p2y = points[1]
            points = []
            api.draw_line(
                "genjix",
                p1x, p1y, p2x, p2y, 0.01,
                1.0, 0.0, 0.0, 1.0
            )
    print(f"Mouse button down: {ev.button} @ ({ev.x}, {ev.y})")

