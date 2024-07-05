#!/usr/bin/env python3

import can

bus = can.interface.Bus(interface='socketcand', host="192.168.0.16", port=29536, channel="can0")

try:
  while True:
    msg = bus.recv()
    print(msg)
except KeyboardInterrupt:
    bus.shutdown()
