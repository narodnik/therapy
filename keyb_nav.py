#!/usr/bin/python
# Keyboard navigation
import asyncio
from pytherapy import (api, run_async_tasks)

async def pan_view():
    async for (keycode, keymods, repeat) in api.key_down:
        #print(f"Key '{keycode}' pressed (keymods={keymods}, repeat={repeat})")
        match keycode:
            case "Left":
                await api.pan(0.01, 0.)
            case "Right":
                await api.pan(-0.01, 0.)
            case "Up":
                await api.pan(0., -0.01)
            case "Down":
                await api.pan(0., 0.01)

async def wheel_zoom():
    async for (x, y) in api.mouse_wheel:
        await api.zoom(1 + y/10)
        print(f"Mouse wheel: ({x}, {y})")

async def say_hello():
    print("calling say hello")
    hello = await api.say_hello()
    print(hello)

run_async_tasks([pan_view(), wheel_zoom(), say_hello()])
