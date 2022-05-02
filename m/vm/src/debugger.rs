use {
    super::{te, Vm},
    std::{
        fmt,
        fs::File,
        io,
        sync::mpsc::{self, Receiver},
        thread::{self, JoinHandle},
    },
};

error::Error! {
    Send = SendError
    Recv = RecvError
    Io = io::Error
    Vm = Box<super::Error>
    Any = String
}

type SendItem = Command;
type SendError = mpsc::SendError<SendItem>;
type RecvError = mpsc::RecvError;
type Recv = Receiver<SendItem>;

#[derive(Default)]
pub struct Callbacks {
    pub data: Vec<Box<dyn FnMut(&mut Vm, &dyn fmt::Debug) -> Result<()>>>,
}

#[derive(Debug)]
pub struct Bugger {
    pub receiver_thread: JoinHandle<Result<()>>,
    pub callbacks: Callbacks,
    recv: Recv,
    in_system_main: bool,
}

#[derive(Debug)]
pub enum Command {
    Echo(String),
}

impl Bugger {
    pub fn run<I>(&mut self, vm: &mut Vm, instr: I) -> Result<()>
    where
        I: fmt::Debug,
    {
        let Self {
            recv,
            in_system_main,
            ..
        } = self;

        let instr_id = vm.instr_addr();

        if !*in_system_main {
            if instr_id == 2 {
                *in_system_main = true;
            }
        } else {
            te!(recv.recv());
            for cb in &mut self.callbacks.data {
                te!(cb.as_mut()(vm, &instr));
            }
            //te!(vm.write_to(Ok(io::stderr())).map_err(Box::new));
            //eprintln!("");
            //eprintln!("");
            //eprintln!("");
            //eprintln!("===== ===== =====");
            //eprintln!("[BUGGER] {} {:?}", instr_id, instr);
        }
        Ok(())
    }
    pub fn open() -> Result<Self> {
        const PATH: &str = "_.xs-debug.sock";
        let (send, recv) = mpsc::channel();
        let sock = te!(File::open(PATH), "Debugger socket: {}", PATH);
        let mut sock = io::BufReader::new(sock);
        let mut buffer = String::new();

        Ok(Bugger {
            recv,
            in_system_main: false,
            callbacks: <_>::default(),
            receiver_thread: thread::spawn(move || {
                use io::BufRead;
                loop {
                    if te!(sock.read_line(&mut buffer)) == 0 {
                        break;
                    }
                    let __ = send.send(Command::Echo(buffer.to_owned()));
                    te!(__);
                }
                Ok(())
            }),
        })
    }
}

impl fmt::Debug for Callbacks {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "callb({})", self.data.len())
    }
}
