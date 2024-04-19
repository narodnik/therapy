import python_libinput

li = python_libinput.libinput()
assert li.start()

while True:
    events = li.poll()
    for event in events:
        # tip up / down
        if event.type == 0:
            if event.tip_is_down:
                print("down")
            else:
                print("up")
        # cursor move
        elif event.type == 1:
            x, y = event.x, event.y
            print(x, y)

