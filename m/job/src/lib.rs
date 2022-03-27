pub const VERSION: &str = "0.0.1";

use {
    error::{te, temg},
    std::{
        borrow::{Borrow, BorrowMut},
        fmt, io, mem,
        process::{Child, Command, ExitStatus, Stdio},
        string,
    },
};

error::Error! {
    Msg = String
    Io = io::Error
    Utf8 = string::FromUtf8Error
}

either::either! {
    #[derive(Debug)]
    pub Job,
        Cmd,
        BindInput,
        Output,
        String,
        Null,
        Cleanups,
        Child
}
either::either! {
    #[derive(Debug)]
    pub Cleanup,
        Thread,
        Child
}
pub type Cmd = Command;
pub type Output = Vec<u8>;
pub type Null = ();
pub type Cleanups = (Command, Child, Vec<Cleanup>);
pub type Thread = std::thread::JoinHandle<Result<()>>;

either::name![ #[derive(Debug)] pub BindInput = (Command, Vec<Job>) ];
either::name![ #[derive(Debug)] pub CleanupChild = Child            ];

impl Job {
    pub fn cleanup(&mut self) -> Result<()> {
        error::ltrace!("cleanup");
        match mem::take(self) {
            Self::Cmd(mut cmd) => {
                cmd.stderr(Stdio::inherit());
                cmd.stdout(Stdio::inherit());

                let mut child = te!(cmd.spawn(), "Spawn {:?}", cmd);

                let status = te!(child.wait());
                if !status.success() {
                    temg!("Subprocess {:?} failed: {:?}", cmd, status)
                }
            }
            Self::BindInput(BindInput((mut cmd, inps))) => match te!(inps.into_iter().next()) {
                Self::Output(bytes) => {
                    error::ldebug!("Writing output");
                    cmd.stdin(Stdio::piped());
                    let mut child = te!(cmd.spwn());
                    let mut stdin = te!(child.stdin.take());
                    let thread_h = std::thread::spawn(move || -> Result<()> {
                        te!(io::Write::write_all(&mut stdin, &bytes));
                        Ok(())
                    });
                    return Self::Cleanups((cmd, child, vec![Cleanup::Thread(thread_h).into()]))
                        .cleanup();
                }
                Self::Child(mut inp_child) => {
                    cmd.stdin(te!(inp_child.stdout.take()));
                    let child = te!(cmd.spwn());
                    return Self::Cleanups((cmd, child, vec![Cleanup::Child(inp_child).into()]))
                        .cleanup();
                }
                other => panic!("{:?}", other),
            },
            Self::Cleanups((cmd, child, cleanups)) => {
                for cleanup in cleanups {
                    match cleanup {
                        Cleanup::Child(mut child) => {
                            let status = te!(child.wait());
                            if !status.success() {
                                temg!("Child {:?} failed: {:?}", child, status)
                            }
                        }
                        other => panic!("{:?}", other),
                    }
                }
                te!(cmd.wait(child));
            }
            other => panic!("{:?}", other),
        }
        Ok(())
    }

    pub fn collect(&mut self) -> Result<()> {
        error::ltrace!("collect");
        match self {
            Self::Cmd(cmd) => {
                cmd.stderr(Stdio::inherit());
                cmd.stdout(Stdio::piped());

                let child = te!(cmd.spawn(), "Spawn {:?}", cmd);

                let output = child.wait_with_output().unwrap();

                let status = output.status;
                if !status.success() {
                    temg!("Subprocess {:?} failed: {:?}", cmd, status)
                }

                *self = output.stdout.into();
            }
            other => panic!("{:?}", other),
        }
        Ok(())
    }

    pub fn pipe(&mut self) -> Result<()> {
        error::ltrace!("collect");
        match self {
            Self::Cmd(cmd) => {
                cmd.stderr(Stdio::inherit());
                cmd.stdout(Stdio::piped());

                let child = te!(cmd.spawn(), "Spawn {:?}", cmd);

                *self = child.into();
            }
            other => panic!("{:?}", other),
        }
        Ok(())
    }

    pub fn make_string(&mut self) -> Result<&str> {
        Ok(match self {
            Self::Output(bytes) => {
                let bytes = mem::take(bytes);
                let string = te!(String::from_utf8(bytes));
                *self = string.into();
                te!(self.make_string())
            }
            Self::String(s) => s.as_str(),
            other => panic!("{:?}", other),
        })
    }

    pub fn add_input_job(&mut self, job: Job) -> Result<()> {
        *self = match mem::take(self) {
            Self::Cmd(cmd) => BindInput((cmd, vec![job])).into(),
            other => panic!("{:?}", other),
        };
        Ok(())
    }
}

impl Default for Job {
    fn default() -> Self {
        Self::Null(())
    }
}

trait CommandExt: Borrow<Command> + BorrowMut<Command> + fmt::Debug {
    fn cmd(&mut self) -> &mut Command {
        self.borrow_mut()
    }
    fn cmd_ref(&self) -> &Command {
        self.borrow()
    }

    fn spwn(&mut self) -> Result<Child> {
        let cmd = self.cmd();
        Ok(te!(cmd.spawn(), "Spawn {:?}", cmd))
    }

    fn wait(&self, mut child: Child) -> Result<ExitStatus> {
        let cmd = self.cmd_ref();
        let status = te!(child.wait(), "Wait {:?}", self);
        if !status.success() {
            temg!("Subprocess {:?} failed: {:?}", cmd, status)
        }
        Ok(status)
    }
}
impl<S> CommandExt for S
where
    S: Borrow<Command>,
    S: BorrowMut<Command>,
    S: fmt::Debug,
{
}
