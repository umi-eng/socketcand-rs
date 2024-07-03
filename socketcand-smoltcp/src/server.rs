use crate::Port;
use core::{fmt::Write, str::from_utf8, task::Waker};
use embedded_can::Frame;
use heapless::String;
use smoltcp::{
    iface::{SocketHandle, SocketSet},
    socket::tcp::{RecvError, SendError, Socket, State},
    time::Instant,
};
use socketcand::{
    wire::{command, Command},
    Mode,
};

/// State container for a connection.
///
/// This is reset to its default value when the client disconnects, ready for
/// the next connection.
#[derive(Debug)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
pub struct ConnectionState {
    /// has the < hi > welcome response been sent
    welcome: bool,
    mode: Mode,
}

impl Default for ConnectionState {
    fn default() -> Self {
        ConnectionState {
            welcome: false,
            mode: Mode::NoBus,
        }
    }
}

/// Socketcand server.
#[derive(Debug)]
pub struct Server {
    socket: SocketHandle,
    port: u16,
    state: ConnectionState,
}

impl Server {
    /// Creates a new socketcand server.
    pub fn new<'a>(
        sockets: &mut SocketSet<'a>,
        socket: Socket<'a>,
        port: Port,
    ) -> Self {
        let handle = sockets.add(socket);

        Self {
            socket: handle,
            port: port.0,
            state: ConnectionState::default(),
        }
    }

    /// Perform socket lifecycle actions.
    fn handle_socket(&mut self, socket: &mut Socket) {
        if !socket.is_open() {
            if !socket.is_listening() {
                socket.listen(self.port).ok();
            }
        }

        // client has disconnected
        if socket.state() == State::CloseWait {
            socket.close();
            // reset internal state
            self.state = ConnectionState::default();
            return;
        }

        if self.state.welcome == false && socket.can_send() {
            // welcome message to client
            socket.send_slice("< hi >".as_bytes()).ok();
            self.state.welcome = true;
        }
    }

    /// Register a waker for receive operations.
    ///
    /// See smoltcp docs for more details.
    pub fn register_recv_waker(
        &mut self,
        sockets: &mut SocketSet,
        waker: &Waker,
    ) {
        let socket = sockets.get_mut::<Socket>(self.socket);

        socket.register_recv_waker(waker);
    }

    /// Receive a command over the connection if there is any.
    pub fn recv<'a>(
        &'a mut self,
        sockets: &'a mut SocketSet,
    ) -> Result<Option<Command>, RecvError> {
        let socket = sockets.get_mut::<Socket>(self.socket);

        self.handle_socket(socket);

        if !socket.can_recv() || !socket.can_send() {
            return Ok(None);
        }

        let cmd = socket
            .recv(|data| {
                match from_utf8(data) {
                    Ok(ascii) => match command(ascii) {
                        Ok((remainder, cmd)) => {
                            let taken = data.len() - remainder.len();
                            return (taken, Some(cmd));
                        }
                        Err(err) => {
                            #[cfg(feature = "defmt-03")]
                            defmt::error!(
                                "Failed to parse command: {}",
                                defmt::Debug2Format(&err),
                            );

                            // clear receive buffer
                            return (data.len(), None);
                        }
                    },
                    Err(err) => {
                        #[cfg(feature = "defmt-03")]
                        defmt::error!(
                            "Failed to convert command to utf8: {}",
                            defmt::Debug2Format(&err)
                        );

                        // clear receive buffer
                        return (data.len(), None);
                    }
                }
            })?
            .clone();

        if let Some(ref cmd) = cmd {
            match cmd {
                Command::Open(_) => {
                    self.state.mode = Mode::Broadcast;
                    socket.send_slice("< ok >".as_bytes()).ok();
                }
                Command::RawMode(_) => {
                    self.state.mode = Mode::Raw;
                    socket.send_slice("< ok >".as_bytes()).ok();
                }
                _ => {}
            }
        }

        Ok(cmd)
    }

    /// Send a CAN frame.
    pub fn send_frame(
        &mut self,
        sockets: &mut SocketSet,
        now: Instant,
        frame: &impl Frame,
    ) -> Result<(), SendError> {
        let socket = sockets.get_mut::<Socket>(self.socket);

        self.handle_socket(socket);

        if socket.may_send() && self.state.mode == Mode::Raw {
            let mut out = String::<128>::new();

            write!(&mut out, "< frame ",).unwrap();

            match frame.id() {
                embedded_can::Id::Standard(id) => {
                    write!(&mut out, "{:03X} ", id.as_raw()).unwrap()
                }
                embedded_can::Id::Extended(id) => {
                    write!(&mut out, "{:08X} ", id.as_raw()).unwrap()
                }
            };

            write!(&mut out, "{}.{} ", now.secs(), now.millis()).unwrap();

            for byte in frame.data() {
                write!(&mut out, "{:02X}", byte).unwrap();
            }

            write!(&mut out, " >").unwrap();

            socket.send_slice(out.as_bytes())?;
        }

        Ok(())
    }
}
