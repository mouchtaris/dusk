pub const VERSION: &str = "0.0.1";

use {
    error::{te, temg},
    std::{io, process::Child},
};

#[derive(Debug)]
pub struct Job {
    core: Core,
    cleanup: Cleanup,
}

#[derive(Debug)]
pub enum Core {
    Proc(Child),
}

#[derive(Debug)]
pub enum Cleanup {
    Inherit,
}

error::Error! {
    Msg = String
    Io = io::Error
}

impl Job {
    pub fn proc(proc: Child) -> Self {
        Self {
            core: Core::Proc(proc),
            cleanup: Cleanup::Inherit,
        }
    }
    pub fn cleanup(&mut self) -> Result<()> {
        error::ldebug!("cleanup");
        match (&mut self.core, &self.cleanup) {
            (Core::Proc(proc), Cleanup::Inherit) => {
                let status = te!(proc.wait());

                if !status.success() {
                    temg!("Subprocess failed: {:?}", status)
                }

                error::soft_todo!();
            }
        }
        Ok(())
    }
    pub fn collect(&mut self) -> Result<()> {
        error::lwarn!("collect");
        match (&mut self.core, &self.cleanup) {
            (Core::Proc(proc), Cleanup::Inherit) => {
                let status = te!(proc.wait());

                if !status.success() {
                    temg!("Subprocess failed: {:?}", status)
                }

                error::soft_todo!();
            }
        }
        Ok(())
    }
}

impl From<Child> for Job {
    fn from(proc: Child) -> Self {
        Self::proc(proc)
    }
}
