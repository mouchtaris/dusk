pub const VERSION: &str = "0.0.1";

use {
    error::{te, temg},
    std::{
        io, mem,
        process::{Command, Stdio},
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
        Output,
        String
}
pub type Cmd = Command;
pub type Output = Vec<u8>;

impl Job {
    pub fn cleanup(&mut self) -> Result<()> {
        error::ltrace!("cleanup");
        match self {
            Self::Cmd(cmd) => {
                cmd.stderr(Stdio::inherit());
                cmd.stdout(Stdio::inherit());
                cmd.stdin(Stdio::inherit());

                let mut child = te!(cmd.spawn(), "Spawn {:?}", cmd);

                let status = te!(child.wait());
                if !status.success() {
                    temg!("Subprocess {:?} failed: {:?}", cmd, status)
                }
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
                cmd.stdin(Stdio::inherit());

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
}
