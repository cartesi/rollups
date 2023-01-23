use super::Context;

use crate::machine::BrokerSend;

use state_fold_types::{ethereum_types::Address, Block};
use types::foldables::input_box::{DAppInputBox, Input, InputBox};

use anyhow::Result;

use tracing::{debug, instrument, trace};

#[derive(Debug)]
pub struct MachineDriver {
    dapp_address: Address,
}

impl MachineDriver {
    pub fn new(dapp_address: Address) -> Self {
        Self { dapp_address }
    }

    #[instrument(level = "trace", skip_all)]
    pub async fn react(
        &self,
        context: &mut Context,
        block: &Block,
        input_box: &InputBox,
        broker: &impl BrokerSend,
    ) -> Result<()> {
        let dapp_input_box =
            match input_box.dapp_input_boxes.get(&self.dapp_address) {
                None => {
                    debug!("No inputs for dapp {}", self.dapp_address);
                    return Ok(());
                }

                Some(d) => d,
            };

        self.process_dapp_inputs(context, dapp_input_box, broker)
            .await?;

        context
            .finish_epoch_if_needed(block.timestamp.as_u64(), broker)
            .await?;

        Ok(())
    }
}

impl MachineDriver {
    #[instrument(level = "trace", skip_all)]
    async fn process_dapp_inputs(
        &self,
        context: &mut Context,
        dapp_input_box: &DAppInputBox,
        broker: &impl BrokerSend,
    ) -> Result<()> {
        trace!(
            "Last input sent to machine manager `{}`, current input `{}`",
            context.inputs_sent_count(),
            dapp_input_box.inputs.len()
        );

        let input_slice = dapp_input_box
            .inputs
            .skip(context.inputs_sent_count() as usize);

        for input in input_slice {
            self.process_input(context, &input, broker).await?;
        }

        Ok(())
    }

    #[instrument(level = "trace", skip_all)]
    async fn process_input(
        &self,
        context: &mut Context,
        input: &Input,
        broker: &impl BrokerSend,
    ) -> Result<()> {
        let input_timestamp = input.block_added.timestamp.as_u64();
        trace!(?context, ?input_timestamp);

        context
            .finish_epoch_if_needed(input_timestamp, broker)
            .await?;

        context.enqueue_input(input, broker).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use state_fold_types::ethereum_types::H160;
    use types::foldables::input_box::Input;

    use crate::drivers::{mock, Context};

    use super::MachineDriver;

    #[tokio::test]
    async fn test_process_input() {
        let dapp_address = H160::random();
        let machine_driver = MachineDriver::new(dapp_address);

        let broker = mock::Broker::new(vec![], vec![]);
        let context = Context::new(0, 5, &broker).await.unwrap();
        let input = Input {
            sender: todo!(),
            payload: todo!(),
            block_added: todo!(),
            dapp: todo!(),
        };

        let result =
            machine_driver.process_input(&mut context, &input, &broker);
        assert!(false)
    }
}
/*
pub struct Context {
    inputs_sent_count: u64,
    last_event_is_finish_epoch: bool,
    last_timestamp: u64,

    // constants
    genesis_timestamp: u64,
    epoch_length: u64,
}
*/
