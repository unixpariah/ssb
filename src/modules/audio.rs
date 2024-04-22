use std::{cell::RefCell, rc::Rc};

use libpulse_binding as pulse;

use pulse::callbacks::ListResult;
use pulse::context::introspect;
use pulse::operation::State;
use pulse::proplist::Proplist;
use pulse::{
    context::{introspect::Introspector, Context},
    mainloop::standard::{IterateResult, Mainloop},
    operation::Operation,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct AudioSettings {
    pub formatting: String,
    #[serde(default)]
    pub icons: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
struct DeviceInfo {
    index: u32,
    volume: pulse::volume::ChannelVolumes,
}

impl From<&introspect::SinkInfo<'_>> for DeviceInfo {
    fn from(info: &introspect::SinkInfo) -> Self {
        DeviceInfo {
            index: info.index,
            volume: info.volume,
        }
    }
}

struct Handler {
    pub mainloop: Mainloop,
    pub context: Context,
    pub introspect: Introspector,
}

impl Drop for Handler {
    fn drop(&mut self) {
        self.context.disconnect();
        self.mainloop.quit(pulse::def::Retval(0));
    }
}

impl Handler {
    fn new() -> Result<Self, Box<dyn crate::Error>> {
        let mut proplist = Proplist::new().ok_or("")?;
        proplist
            .set_str(
                pulse::proplist::properties::APPLICATION_NAME,
                "SinkController",
            )
            .map_err(|_| "")?;

        let mut mainloop = Mainloop::new().ok_or("")?;
        let mut context = Context::new_with_proplist(&mainloop, "MainConn", &proplist).ok_or("")?;
        context.connect(None, pulse::context::FlagSet::NOFLAGS, None)?;

        loop {
            match mainloop.iterate(false) {
                IterateResult::Err(e) => {
                    return Err(e.into());
                }
                IterateResult::Success(_) => {}
                IterateResult::Quit(_) => {
                    return Err("Iterate state quit without an error".into());
                }
            }

            match context.get_state() {
                pulse::context::State::Ready => break,
                pulse::context::State::Failed | pulse::context::State::Terminated => {
                    return Err("Context state failed/terminated without an error".into());
                }
                _ => {}
            }
        }

        let introspect = context.introspect();

        Ok(Handler {
            mainloop,
            context,
            introspect,
        })
    }

    pub fn wait_for_operation<G: ?Sized>(
        &mut self,
        op: Operation<G>,
    ) -> Result<(), Box<dyn crate::Error>> {
        loop {
            match self.mainloop.iterate(false) {
                IterateResult::Err(e) => return Err(e.into()),
                IterateResult::Success(_) => {}
                IterateResult::Quit(_) => {}
            }
            match op.get_state() {
                State::Done => {
                    break;
                }
                State::Running => {}
                State::Cancelled => {
                    return Err("Operation cancelled without an error".into());
                }
            }
        }
        Ok(())
    }

    fn get_server_info(&mut self) -> Result<String, Box<dyn crate::Error>> {
        let server: Rc<RefCell<Option<Option<String>>>> = Rc::new(RefCell::new(Some(None)));
        {
            let server = server.clone();
            let op = self.introspect.get_server_info(move |res| {
                server
                    .borrow_mut()
                    .replace(res.default_sink_name.as_ref().map(|cow| cow.to_string()));
            });
            self.wait_for_operation(op)?;
        }

        let result = server.borrow_mut().take().flatten();
        result.ok_or("".into())
    }

    fn get_device_by_name(&mut self, name: &str) -> Result<u32, Box<dyn crate::Error>> {
        let device = Rc::new(RefCell::new(None));
        {
            let dev = device.clone();
            let op = self.introspect.get_sink_info_by_name(
                name,
                move |sink_list: ListResult<&introspect::SinkInfo>| {
                    if let ListResult::Item(item) = sink_list {
                        dev.borrow_mut().replace(item.index);
                    }
                },
            );
            self.wait_for_operation(op)?;
        }
        let mut result = device.borrow_mut();
        result.take().ok_or("".into())
    }

    fn get_default_device(&mut self) -> Result<u32, Box<dyn crate::Error>> {
        let server_info = self.get_server_info();
        match server_info {
            Ok(info) => self.get_device_by_name(&info),
            Err(e) => Err(e),
        }
    }

    fn list_devices(&mut self) -> Result<Vec<DeviceInfo>, Box<dyn crate::Error>> {
        let list = Rc::new(RefCell::new(Vec::new()));
        {
            let list = list.clone();
            let op = self.introspect.get_sink_info_list(
                move |sink_list: ListResult<&introspect::SinkInfo>| {
                    if let ListResult::Item(item) = sink_list {
                        list.borrow_mut().push(item.into());
                    }
                },
            );
            self.wait_for_operation(op)?;
        }
        let result = list.borrow_mut().to_vec();
        Ok(result)
    }
}

pub fn audio() -> Result<String, Box<dyn crate::Error>> {
    let mut handler = Handler::new()?;
    let default_device_index = handler.get_default_device()?;
    let devices = handler.list_devices()?;

    devices
        .iter()
        .find(|dev| dev.index == default_device_index)
        .map(|dev| {
            dev.volume
                .print()
                .split_whitespace()
                .nth(1)
                .unwrap_or("")
                .replace('%', "")
        })
        .ok_or_else(|| "".into())
}
