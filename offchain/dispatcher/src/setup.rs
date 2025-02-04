// Copyright Cartesi Pte. Ltd.
//
// SPDX-License-Identifier: Apache-2.0
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

use rollups_events::DAppMetadata;
use snafu::ResultExt;
use state_client_lib::{
    config::SCConfig, error::StateServerError, BlockServer,
    GrpcStateFoldClient, StateServer,
};
use state_fold_types::BlockStreamItem;
use tokio_stream::{Stream, StreamExt};
use tonic::transport::Channel;
use types::foldables::authority::{RollupsInitialState, RollupsState};

use crate::{
    config::DispatcherConfig,
    drivers::Context,
    error::{
        BrokerSnafu, ChannelSnafu, ConnectSnafu, DispatcherError,
        StateServerSnafu,
    },
    machine::BrokerStatus,
    metrics::DispatcherMetrics,
};

const BUFFER_LEN: usize = 256;

pub async fn create_state_server(
    config: &SCConfig,
) -> Result<
    impl StateServer<InitialState = RollupsInitialState, State = RollupsState>
        + BlockServer,
    DispatcherError,
> {
    let channel = Channel::from_shared(config.grpc_endpoint.to_owned())
        .context(ChannelSnafu)?
        .connect()
        .await
        .context(ConnectSnafu)?;

    Ok(GrpcStateFoldClient::new_from_channel(channel))
}

pub async fn create_block_subscription(
    client: &impl BlockServer,
    confirmations: usize,
) -> Result<
    impl Stream<Item = Result<BlockStreamItem, StateServerError>>
        + Send
        + std::marker::Unpin,
    DispatcherError,
> {
    let s = client
        .subscribe_blocks(confirmations)
        .await
        .context(StateServerSnafu)?;

    let s = {
        use futures::StreamExt;
        s.ready_chunks(BUFFER_LEN)
    };

    let s = s.filter_map(
        |mut x| {
            if x.len() == BUFFER_LEN {
                None
            } else {
                x.pop()
            }
        },
    );

    Ok(s)
}

pub async fn create_context(
    config: &DispatcherConfig,
    block_server: &impl BlockServer,
    broker: &impl BrokerStatus,
    dapp_metadata: DAppMetadata,
    metrics: DispatcherMetrics,
) -> Result<Context, DispatcherError> {
    let genesis_timestamp: u64 = block_server
        .query_block(config.dapp_deployment.deploy_block_hash)
        .await
        .context(StateServerSnafu)?
        .timestamp
        .as_u64();
    let epoch_length = config.epoch_duration;
    let context = Context::new(
        genesis_timestamp,
        epoch_length,
        broker,
        dapp_metadata,
        metrics,
    )
    .await
    .context(BrokerSnafu)?;

    Ok(context)
}
