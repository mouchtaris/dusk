pub const VERSION: &str = "0.0.1";

use {
    error::{ldebug, te, temg},
    std::{
        fmt, io, mem,
        process::{Child, Command, ExitStatus, Stdio},
        thread::JoinHandle,
    },
};

error::Error! {
    Msg = String
    Io = io::Error
    Thread = Box<dyn std::any::Any + Send>
    Utf8 = std::string::FromUtf8Error
}

either::either![
    #[derive(Debug)]
    pub Job,
        Null,
        Spec,
        System,
        Buffer
];

#[derive(Debug)]
pub struct Spec {
    pub cmd: Command,
    pub input: Vec<Job>,
}

#[derive(Debug)]
pub enum SystemItem {
    Child(Child),
    Buffer(Vec<u8>),
}

#[derive(Debug)]
pub struct System {
    pub cmd: Command,
    pub item: SystemItem,
    pub cleanup: Vec<Cleanup>,
    pub init: Vec<Init>,
}

pub struct Init(Box<dyn FnMut(&mut Child) -> Result<Cleanup>>);

#[derive(Debug)]
pub enum Cleanup {
    Child(Child, Command),
    Thread(Thread),
}

#[derive(Debug)]
pub enum Buffer {
    Null,
    Bytes(Command, Vec<u8>),
    String(Command, String),
}

pub type Thread = JoinHandle<Result<()>>;

impl From<Command> for Job {
    fn from(cmd: Command) -> Self {
        Self::Spec(Spec {
            cmd,
            input: <_>::default(),
        })
    }
}

impl Default for Job {
    fn default() -> Self {
        Self::Null(())
    }
}

pub type Null = ();

impl Job {
    pub fn as_spec_mut(&mut self) -> Result<&mut Spec> {
        Ok(match self {
            Self::Spec(spec) => spec,
            other => temg!("Internal error: not a spec job: {:?}", other),
        })
    }

    pub fn as_buffer_mut(&mut self) -> Result<&mut Buffer> {
        Ok(match self {
            Self::Buffer(buf) => buf,
            other => te!(other.made_buffer().and_then(Self::as_buffer_mut)),
        })
    }

    pub fn as_buffer(&self) -> Result<&Buffer> {
        Ok(match self {
            Self::Buffer(buf) => buf,
            other => temg!("Cannot be a buffer: {:?}", other),
        })
    }

    pub fn add_input_job(&mut self, input_job: &mut Job) -> Result<()> {
        self.as_spec_mut().map(|Spec { input, .. }| {
            let input_job = match input_job {
                Job::Null(_) => Job::Null(()),
                Job::Buffer(Buffer::Null) => Job::Buffer(Buffer::Null),

                Job::Buffer(Buffer::Bytes(_, buf))
                | Job::System(System {
                    item: SystemItem::Buffer(buf),
                    ..
                }) => echo_buffer_job(buf),

                Job::Buffer(Buffer::String(_, _)) |
                Job::System(System {
                    item: SystemItem::Child(_),
                    ..
                })
                | Job::Spec(_) => mem::take(input_job),
            };
            input.push(input_job)
        })
    }

    pub fn as_bytes(&self) -> Result<&[u8]> {
        Ok(te!(self.as_buffer()).as_bytes())
    }

    pub fn as_str(&self) -> Result<&str> {
        Ok(te!(te!(self.as_buffer()).as_str()))
    }

    pub fn into_pipe(self, capture: bool) -> Result<System> {
        Ok(match self {
            Self::Spec(spec) => te!(spawn_spec(spec, capture)),
            Self::System(s) => s,
            Self::Buffer(buf) => echo_buffer(buf, capture),
            Self::Null(_) => temg!("Cannot pipe null Job"),
        })
    }
    pub fn make_pipe(&mut self, capture: bool) -> Result<()> {
        Ok(*self = te!(mem::take(self).into_pipe(capture)).into())
    }

    pub fn into_buffer(self) -> Result<Buffer> {
        Ok(match self {
            Self::Spec(s) => te!(collect_output(te!(spawn_spec(s, true)))),
            Self::System(sys) => te!(collect_output(sys)),
            other => panic!("{:?}", other),
        })
    }

    pub fn made_buffer(&mut self) -> Result<&mut Self> {
        te!(self.make_buffer());
        Ok(self)
    }
    pub fn make_buffer(&mut self) -> Result<()> {
        Ok(*self = te!(mem::take(self).into_buffer()).into())
    }

    pub fn make_string(&mut self) -> Result<&str> {
        Ok(te!(te!(self.as_buffer_mut()).make_string()))
    }

    pub fn cleanup(&mut self) -> Result<()> {
        ldebug!("cleanup {:?}", self);
        let job = mem::take(self);
        let sys = te!(job.into_pipe(false));
        te!(sys.cleanup());
        Ok(())
    }
    pub fn collect(&mut self) -> Result<()> {
        self.make_buffer()
    }
    pub fn pipe(&mut self) -> Result<()> {
        self.make_pipe(true)
    }
}

fn echo_buffer_job<B: Byteable + ToOwned<Owned = B>>(buf: &B) -> Job {
    Job::System(echo_buffer(
        Buffer::Bytes(
            Command::new("<internal input job>"),
            buf.to_owned().into_bytes(),
        ),
        true,
    ))
}

fn echo_buffer(buf: Buffer, capture: bool) -> System {
    let cmd = Command::new("<internal pipe>");

    let sys = System {
        cmd,
        item: SystemItem::Buffer(buf.take_bytes()),
        cleanup: <_>::default(),
        init: <_>::default(),
    };

    ldebug!(
        "Echo {}buffer: {:?}",
        if capture { "capturing " } else { "" },
        sys
    );
    sys
}

fn collect_output(sys: System) -> Result<Buffer> {
    let System {
        cmd, item, cleanup, ..
    } = sys;

    let stdout = match item {
        SystemItem::Child(child) => {
            let output = te!(child.wait_with_output());

            te!(check_exit_status(&cmd, output.status));
            te!(Cleanup::all(cleanup));

            output.stdout
        }
        SystemItem::Buffer(buf) => buf,
    };

    let buffer = Buffer::Bytes(cmd, stdout);

    ldebug!("Collect output: {:?}", buffer);
    Ok(buffer)
}

fn connect_input(cmd: &mut Command, input: Job) -> Result<System> {
    let mut inp_sys = te!(input.into_pipe(true));

    match &mut inp_sys.item {
        SystemItem::Child(child) => {
            let inp_stdout = te!(child.stdout.take(), "Missing output of {:?}", cmd);
            cmd.stdin(inp_stdout);
        }
        SystemItem::Buffer(ref buf) => {
            cmd.stdin(Stdio::piped());
            let buf = buf.to_owned();
            inp_sys.init.push(Init(Box::new(move |child| {
                let buf = buf.to_owned(); //?
                Ok({
                    let mut stdin = te!(child.stdin.take(), "Missing stdin on child {:?}", child);
                    let thread = std::thread::spawn(move || -> Result<()> {
                        te!(io::Write::write_all(&mut stdin, &buf));
                        Ok(())
                    });
                    Cleanup::Thread(thread)
                })
            })));
        }
    }

    ldebug!("Connect input: {:?}", inp_sys);
    Ok(inp_sys)
}
fn spawn_spec(Spec { mut cmd, input }: Spec, capture: bool) -> Result<System> {
    let mut cleanup: Vec<Cleanup> = <_>::default();

    let mut inp_inits = Vec::new();
    for input in input {
        let inp_sys = te!(connect_input(&mut cmd, input));
        let (inp_init, inp_cleanup) = inp_sys.into_init_cleanup();
        inp_inits.extend(inp_init);
        cleanup.extend(inp_cleanup);
    }

    //cmd.stdin(Stdio::piped());
    if capture {
        cmd.stdout(Stdio::piped());
    } else {
        cmd.stdout(Stdio::inherit());
    }

    let mut child = te!(cmd.spawn(), "Spawning {:?}", cmd);
    for init in &mut inp_inits {
        cleanup.push(te!(init.0(&mut child)));
    }

    let sys = System {
        cmd,
        item: SystemItem::Child(child),
        cleanup,
        init: <_>::default(),
    };

    ldebug!(
        "Spawn {}capturing command {:?}",
        if capture { "" } else { "non-" },
        sys
    );
    Ok(sys)
}

fn check_exit_status(cmd: &Command, status: ExitStatus) -> Result<()> {
    if !status.success() {
        temg!("Subprocess {:?} failed: {:?}", cmd, status);
    }
    Ok(())
}

impl Cleanup {
    fn perform(self) -> Result<()> {
        use Cleanup as C;
        Ok(match self {
            C::Child(mut child, cmd) => {
                ldebug!("Child wait {:?}", cmd);
                let status = te!(child.wait());
                te!(check_exit_status(&cmd, status));
            }
            C::Thread(handle) => {
                ldebug!("Thread wait {:?}", handle);
                let thread_result = te!(handle.join());
                let comp_result = te!(thread_result);
                comp_result
            }
        })
    }
    fn all<C>(cleanups: C) -> Result<()>
    where
        C: IntoIterator,
        C::Item: Into<Cleanup>,
    {
        cleanups
            .into_iter()
            .map(<_>::into)
            .map(Cleanup::perform)
            .collect()
    }
}

impl Buffer {
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            Self::Bytes(_, v) => v,
            Self::String(_, v) => v.as_bytes(),
            Self::Null => &[],
        }
    }
    pub fn take_bytes(self) -> Vec<u8> {
        match self {
            Self::Bytes(_, v) => v,
            Self::String(_, v) => v.into_bytes(),
            other => panic!("{:?}", other),
        }
    }
    pub fn as_str(&self) -> Result<&str> {
        Ok(match self {
            Self::String(_, v) => v,
            other => temg!("Cannot &str from {:?}", other),
        })
    }
    pub fn make_string(&mut self) -> Result<&str> {
        Ok(match mem::take(self) {
            Buffer::Null => temg!("Cannot make string from Null"),
            Buffer::String(cmd, string) => {
                // Nothing changes here, just reconstruct because things
                // are moved out of `self`.
                *self = Buffer::String(cmd, string);
                te!(self.as_str())
            }
            Buffer::Bytes(cmd, bytes) => {
                let s = te!(
                    String::from_utf8(bytes),
                    "Convert output of {:?} to String",
                    cmd
                );
                *self = Buffer::String(cmd, s);
                te!(self.make_string())
            }
        })
    }
}

impl Default for Buffer {
    fn default() -> Self {
        Self::Null
    }
}

impl System {
    pub fn into_init_cleanup(self) -> (Vec<Init>, Vec<Cleanup>) {
        let Self {
            cmd,
            item,
            mut cleanup,
            init,
        } = self;
        match item {
            SystemItem::Child(child) => {
                cleanup.push(Cleanup::Child(child, cmd));
            }
            _ => (),
        }
        (init, cleanup)
    }

    pub fn cleanup(self) -> Result<()> {
        ldebug!("cleanup {:?}", self);
        Cleanup::all(self.into_init_cleanup().1)
    }
}

impl fmt::Debug for Init {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Init").finish()
    }
}

pub trait Byteable {
    fn into_bytes(self) -> Vec<u8>;
}
impl Byteable for Vec<u8> {
    fn into_bytes(self) -> Vec<u8> {
        self
    }
}
impl Byteable for String {
    fn into_bytes(self) -> Vec<u8> {
        self.into_bytes()
    }
}
