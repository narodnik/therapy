from sdbus import (DbusInterfaceCommonAsync, dbus_method_async,
                   dbus_property_async, dbus_signal_async,
                   sd_bus_open_user)

class TherapyDbusApi(
    DbusInterfaceCommonAsync,
    interface_name='org.therapy.Therapy'
):
    @dbus_method_async(
        result_signature='s',
    )
    async def say_hello(self) -> str:
        raise NotImplementedError

    @dbus_method_async(
        input_signature='sddddddddd',
    )
    async def draw_line(
        self, layer_name: str,
        x1: float, y1: float, x2: float, y2: float,
        thickness: float,
        r: float, g: float, b: float, a: float
    ):
        raise NotImplementedError

    @dbus_method_async(
        input_signature='dd',
    )
    async def pan(self, x: float, y: float):
        raise NotImplementedError

    @dbus_method_async(
        input_signature='d',
    )
    async def zoom(self, scale):
        raise NotImplementedError

    @dbus_method_async(
        input_signature='dd',
        result_signature='dd',
    )
    async def screen_to_world(self, x: float, y: float) -> (float, float):
        raise NotImplementedError

    @dbus_signal_async(
        signal_signature='sasb'
    )
    def key_down(self, key: str, keymods: list[str], repeat: bool) -> int:
        raise NotImplementedError

    @dbus_signal_async(
        signal_signature='dd'
    )
    def mouse_motion(self, x: float, y: float) -> int:
        raise NotImplementedError

    @dbus_signal_async(
        signal_signature='dd'
    )
    def mouse_wheel(self, x: float, y: float) -> int:
        raise NotImplementedError

    @dbus_signal_async(
        signal_signature='sdd'
    )
    def mouse_button_down(self, button: str, x: float, y: float) -> int:
        raise NotImplementedError

    @dbus_signal_async(
        signal_signature='sdd'
    )
    def mouse_button_up(self, button: str, x: float, y: float) -> int:
        raise NotImplementedError

bus = sd_bus_open_user()
api = TherapyDbusApi.new_proxy("org.therapy.Therapy", "/org/therapy/Therapy", bus)

