pub const VERSION: &str = "0.0.1";

use {
    error::{ldebug, te, temg},
    std::{
        io, mem,
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
pub struct System {
    pub cmd: Command,
    pub child: Child,
    pub cleanup: Vec<Cleanup>,
}

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
            other => temg!("{:?}", other),
        })
    }

    pub fn as_buffer_mut(&mut self) -> Result<&mut Buffer> {
        Ok(match self {
            Self::Buffer(buf) => buf,
            other => temg!("{:?}", other),
        })
    }

    pub fn as_buffer(&self) -> Result<&Buffer> {
        Ok(match self {
            Self::Buffer(buf) => buf,
            other => temg!("{:?}", other),
        })
    }

    pub fn add_input_job(&mut self, input_job: Job) -> Result<()> {
        self.as_spec_mut()
            .map(|Spec { input, .. }| input.push(input_job))
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
            Self::Buffer(buf) => te!(echo_buffer(buf, capture)),
            other => panic!("{:?}", other),
        })
    }
    pub fn make_pipe(&mut self, capture: bool) -> Result<()> {
        Ok(*self = Job::System(te!(mem::take(self).into_pipe(capture))))
    }

    pub fn into_buffer(self) -> Result<Buffer> {
        Ok(match self {
            Self::Spec(s) => te!(collect_output(te!(spawn_spec(s, true)))),
            other => panic!("{:?}", other),
        })
    }

    pub fn make_buffer(&mut self) -> Result<()> {
        Ok(*self = te!(mem::take(self).into_buffer()).into())
    }

    pub fn make_string(&mut self) -> Result<&str> {
        Ok(te!(te!(self.as_buffer_mut()).make_string()))
    }

    pub fn cleanup(&mut self) -> Result<()> {
        self.make_pipe(false)
    }
    pub fn collect(&mut self) -> Result<()> {
        self.make_buffer()
    }
    pub fn pipe(&mut self) -> Result<()> {
        self.make_pipe(true)
    }
}

fn echo_buffer(buf: Buffer, capture: bool) -> Result<System> {
    // TODO internal piping
    let mut cmd = Command::new("cat");

    if capture {
        cmd.stdout(Stdio::piped());
    } else {
        cmd.stdout(Stdio::inherit());
    }
    cmd.stdin(Stdio::piped());

    let mut child = te!(cmd.spawn(), "Spawning {:?}", cmd);
    let mut stdin = te!(child.stdin.take());
    let bytes = buf.take_bytes();

    let thread = std::thread::spawn(move || -> Result<()> {
        te!(io::Write::write_all(&mut stdin, &bytes));
        Ok(())
    });

    let cleanup = vec![Cleanup::Thread(thread)];

    let sys = System {
        cmd,
        child,
        cleanup,
    };

    ldebug!(
        "Echo {}buffer: {:?}",
        if capture { "capturing " } else { "" },
        sys
    );
    Ok(sys)
}

fn collect_output(sys: System) -> Result<Buffer> {
    let System {
        cmd,
        child,
        cleanup,
    } = sys;

    let mut output = te!(child.wait_with_output());

    te!(check_exit_status(&cmd, output.status));
    te!(Cleanup::all(cleanup));

    let stdout = mem::take(&mut output.stdout);

    let buffer = Buffer::Bytes(cmd, stdout);

    ldebug!("Collect output: {:?}", buffer);
    Ok(buffer)
}

fn connect_input(cmd: &mut Command, input: Job) -> Result<System> {
    let mut inp_sys = te!(input.into_pipe(true));

    let inp_stdout = te!(inp_sys.child.stdout.take(), "Missing output of {:?}", cmd);
    cmd.stdin(inp_stdout);

    ldebug!("Connect input: {:?}", inp_sys);
    Ok(inp_sys)
}
fn spawn_spec(Spec { mut cmd, input }: Spec, capture: bool) -> Result<System> {
    let mut cleanup: Vec<Cleanup> = <_>::default();

    for input in input {
        let inp_sys = te!(connect_input(&mut cmd, input));
        let inp_cleanup: Vec<Cleanup> = inp_sys.into();
        cleanup.extend(inp_cleanup);
    }

    //cmd.stdin(Stdio::piped());
    if capture {
        cmd.stdout(Stdio::piped());
    } else {
        cmd.stdout(Stdio::inherit());
    }

    let child = te!(cmd.spawn(), "Spawning {:?}", cmd);

    let sys = System {
        cmd,
        child,
        cleanup,
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
                let status = te!(child.wait());
                te!(check_exit_status(&cmd, status));
            }
            C::Thread(handle) => {
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

impl From<System> for Vec<Cleanup> {
    fn from(
        System {
            cmd,
            child,
            mut cleanup,
        }: System,
    ) -> Self {
        cleanup.push(Cleanup::Child(child, cmd));
        cleanup
    }
}
