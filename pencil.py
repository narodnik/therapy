#!/usr/bin/python
# Mouse pencil tool
import asyncio
from pytherapy import (api, run_async_tasks)

MOUSE_STATE = 0
current_pos = (0., 0.)

async def btn_down():
    global MOUSE_STATE, current_pos
    points = []
    async for (button, x, y) in api.mouse_button_down:
        if button == "Left":
            MOUSE_STATE = 1
            current_pos = (x, y)
        print(f"Mouse button down: {button} @ ({x}, {y})")

async def btn_up():
    global MOUSE_STATE, current_pos
    points = []
    async for (button, x, y) in api.mouse_button_up:
        if button == "Left":
            MOUSE_STATE = 0
            current_pos = (x, y)
        print(f"Mouse button up: {button} @ ({x}, {y})")

async def motion():
    global current_pos
    points = []
    async for (x, y) in api.mouse_motion:
        if MOUSE_STATE == 1:
            x0, y0 = current_pos
            await api.draw_line(
                "genjix",
                x0, y0, x, y, 0.001,
                1.0, 0.0, 0.0, 1.0
            )
            current_pos = (x, y)
        #print(f"Mouse mouse: ({x}, {y})")

run_async_tasks([btn_down(), btn_up(), motion()])


