#!/usr/bin/python
# Mouse pencil tool
import asyncio
from pytherapy import (api, run_async_tasks)
import python_libinput

async def draw():
    MOUSE_STATE = 0
    current_pos = (0., 0.)

    li = python_libinput.libinput()
    assert li.start()

    while True:
        events = li.poll()
        for event in events:
            # tip up / down
            if event.type == 0:
                if event.tip_is_down:
                    MOUSE_STATE = 1
                    x, y = await api.screen_to_world(x, y)
                    current_pos = (x, y)
                else:
                    MOUSE_STATE = 0
                    x, y = await api.screen_to_world(x, y)
                    current_pos = (x, y)
            # cursor move
            elif event.type == 1:
                x, y = event.x, event.y
                #print(x, y)
                if MOUSE_STATE == 1:
                    x0, y0 = current_pos
                    x, y = await api.screen_to_world(x, y)
                    await api.draw_line(
                        "genjix",
                        x0, y0, x, y, 0.001,
                        1.0, 0.0, 0.0, 1.0
                    )
                    current_pos = (x, y)

run_async_tasks([draw()])

