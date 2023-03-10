# Therapy for hackers

This is a collaborative drawing application that allows multiple users to draw on the same canvas in real-time. Users can connect to the application using a web browser and start drawing immediately.

### Requirements

```
C++11 compatible compiler
SFML library
Python 3
pyzmq library
python-libinput library
```

## Building and Running

To build the application, you can use the provided Makefile. Simply navigate to the root directory and run the following command:

```
make
```

To run the application, start the server by running the following command:

```
./server
```

### NOTE: // this is probbably soem crap, not sure how I invented some server bin. Maybe it gets built? who knows 
The server will start running on localhost:8080. Users can connect to the server using a web browser by navigating to localhost:8080/client.

To capture input from a drawing tablet, run the following Python script:

```
python3 input.py
```

This will start capturing input from the drawing tablet and printing it to the console.

## Proxy Server

The application uses a proxy server to enable communication between the web clients and the C++ server. The proxy.py script handles this communication by using ZeroMQ to proxy messages between the web clients and the C++ server.

To run the proxy server, use the following command:

```
proxy.py
```
The proxy server will start listening on tcp://*:5559 and tcp://*:5560 for messages from the web clients and the C++ server, respectively.

License

This project is licensed under the MIT License. See the LICENSE file for more information.
