use std::sync::Arc;
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

#[derive(Deserialize, Serialize, PartialEq)]
pub struct AudioSettings {
    pub formatting: Arc<str>,
    #[serde(default)]
    pub icons: Vec<Box<str>>,
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
    fn new() -> anyhow::Result<Self> {
        let mut proplist = Proplist::new().ok_or_else(|| anyhow::anyhow!(""))?;
        proplist
            .set_str(
                pulse::proplist::properties::APPLICATION_NAME,
                "SinkController",
            )
            .map_err(|_| anyhow::anyhow!(""))?;

        let mut mainloop = Mainloop::new().ok_or_else(|| anyhow::anyhow!(""))?;
        let mut context = Context::new_with_proplist(&mainloop, "MainConn", &proplist)
            .ok_or_else(|| anyhow::anyhow!(""))?;
        context.connect(None, pulse::context::FlagSet::NOFLAGS, None)?;

        loop {
            match mainloop.iterate(true) {
                IterateResult::Err(e) => {
                    return Err(e.into());
                }
                IterateResult::Success(_) => {}
                IterateResult::Quit(_) => {
                    return Err(anyhow::anyhow!("Iterate state quit without an error"));
                }
            }

            match context.get_state() {
                pulse::context::State::Ready => break,
                pulse::context::State::Failed | pulse::context::State::Terminated => {
                    return Err(anyhow::anyhow!(
                        "Context state failed/terminated without an error"
                    ));
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

    fn wait_for_operation<G: ?Sized>(&mut self, op: Operation<G>) -> anyhow::Result<()> {
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
                    return Err(anyhow::anyhow!("Operation cancelled without an error"));
                }
            }
        }
        Ok(())
    }

    fn get_default_device_volume(&mut self) -> anyhow::Result<ChannelVolumes> {
        let server: Rc<RefCell<Option<Box<str>>>> = Rc::new(RefCell::new(None));
        {
            let server = server.clone();
            let op = self.introspect.get_server_info(move |result| {
                *server.borrow_mut() = result
                    .default_sink_name
                    .as_ref()
                    .map(|cow| cow.as_ref().into());
            });
            self.wait_for_operation(op)?;
        }
        let default_sink_name = server
            .borrow_mut()
            .take()
            .ok_or_else(|| anyhow::anyhow!(""))?;

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
        default_device.take().ok_or_else(|| anyhow::anyhow!(""))
    }
}

pub fn audio() -> anyhow::Result<Box<str>> {
    let mut handler = Handler::new()?;
    let default_device_volume = handler.get_default_device_volume()?;
    Ok(default_device_volume
        .print()
        .split_whitespace()
        .nth(1)
        .ok_or_else(|| anyhow::anyhow!(""))?
        .replace('%', "")
        .into())
}
