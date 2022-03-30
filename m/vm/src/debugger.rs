use {
    super::{te, ICode, Vm},
    std::{
        fs::File,
        io,
        sync::mpsc::{self, Receiver},
        thread::{self, JoinHandle},
    },
};

type SendItem = Command;
type SendError = mpsc::SendError<SendItem>;
type RecvError = mpsc::RecvError;
type Recv = Receiver<SendItem>;

#[derive(Debug)]
pub struct Bugger {
    recv: Recv,
    pub receiver_thread: JoinHandle<Result<()>>,
    in_system_main: bool,
}

#[derive(Debug)]
pub enum Command {
    Echo(String),
}

error::Error! {
    Send = SendError
    Recv = RecvError
    Io = io::Error
    Vm = Box<super::Error>
}

impl Bugger {
    pub fn run(&mut self, vm: &mut Vm, icode: &ICode) -> Result<()> {
        let Self {
            recv,
            in_system_main,
            ..
        } = self;

        let instr_id = vm.instr_addr();
        let instr = &icode.instructions[instr_id];

        if !*in_system_main {
            if instr_id == 2 {
                *in_system_main = true;
            }
        } else {
            te!(recv.recv());
            te!(vm.write_to(Ok(io::stderr())).map_err(Box::new));
            eprintln!("");
            eprintln!("===== ===== =====");
            eprintln!("[BUGGER] {} {:?}", instr_id, instr);
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
