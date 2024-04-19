#!/usr/bin/python
# Draw lines by clicking
import asyncio
from pytherapy import (api, run_async_tasks)

async def line_tool():
    points = []
    async for (button, x, y) in api.mouse_button_down:
        if button == "Left":
            points.append((x, y))
            if len(points) == 2:
                p1x, p1y = points[0]
                p2x, p2y = points[1]
                points = []
                await api.draw_line(
                    "genjix",
                    p1x, p1y, p2x, p2y, 0.01,
                    1.0, 0.0, 0.0, 1.0
                )
        print(f"Mouse button down: {button} @ ({x}, {y})")

run_async_tasks([line_tool()])

