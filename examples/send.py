import can
import time

bus = can.interface.Bus(interface='socketcand', host="192.168.0.16", port=29536, channel="can0")

try:
  while True:
    msg = can.Message(arbitration_id=0xc0ffee, data=[0, 1, 2, 3], is_extended_id=True, is_fd=True)
    print(msg)
    bus.send(msg)
    time.sleep(1)
except KeyboardInterrupt:
    bus.shutdown()
