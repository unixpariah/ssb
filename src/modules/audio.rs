use std::{cell::RefCell, rc::Rc};

use libpulse_binding as pulse;

use pulse::callbacks::ListResult;
use pulse::context::introspect::{self};
use pulse::operation::State;
use pulse::proplist::Proplist;
use pulse::volume::ChannelVolumes;
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

struct Handler {
    mainloop: Mainloop,
    context: Context,
    introspect: Introspector,
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
            match mainloop.iterate(true) {
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

    fn wait_for_operation<G: ?Sized>(
        &mut self,
        op: Operation<G>,
    ) -> Result<(), Box<dyn crate::Error>> {
        loop {
            match self.mainloop.iterate(true) {
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

    fn get_default_device_volume(&mut self) -> Result<ChannelVolumes, Box<dyn crate::Error>> {
        let server: Rc<RefCell<Option<Option<String>>>> = Rc::new(RefCell::new(Some(None)));
        {
            let server = server.clone();
            let op = self.introspect.get_server_info(move |result| {
                server
                    .borrow_mut()
                    .replace(result.default_sink_name.as_ref().map(|cow| cow.to_string()));
            });
            self.wait_for_operation(op)?;
        }
        let default_sink_name = server.borrow_mut().take().flatten().ok_or("")?;

        let device = Rc::new(RefCell::new(None));
        {
            let device = device.clone();
            let op = self.introspect.get_sink_info_by_name(
                &default_sink_name,
                move |sink_list: ListResult<&introspect::SinkInfo>| {
                    if let ListResult::Item(item) = sink_list {
                        device.borrow_mut().replace(item.volume);
                    }
                },
            );
            self.wait_for_operation(op)?;
        }
        let mut default_device = device.borrow_mut();
        default_device.take().ok_or("".into())
    }
}

pub fn audio() -> Result<String, Box<dyn crate::Error>> {
    let mut handler = Handler::new()?;
    let default_device_volume = handler.get_default_device_volume()?;
    Ok(default_device_volume
        .print()
        .split_whitespace()
        .nth(1)
        .ok_or("")?
        .replace('%', ""))
}
