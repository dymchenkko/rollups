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

use rollups_data::Repository;
use rollups_events::indexer::{IndexerEvent, IndexerState};
use rollups_events::{
    Broker, BrokerError, RollupsData, RollupsInput, RollupsOutput,
};
use snafu::ResultExt;

use crate::conversions::*;
use crate::error::{
    BrokerSnafu, IndexerError, JoinSnafu, MigrationsSnafu, RepositorySnafu,
};
use crate::IndexerConfig;

pub struct Indexer {
    repository: Repository,
    broker: Broker,
    state: IndexerState,
}

impl Indexer {
    #[tracing::instrument(level = "trace", skip_all)]
    pub async fn start(config: IndexerConfig) -> Result<(), IndexerError> {
        tracing::info!("running database migrations");
        let endpoint = config.repository_config.endpoint();
        rollups_data::run_migrations(&endpoint.into_inner())
            .context(MigrationsSnafu)?;

        tracing::info!("runned migrations; connecting to DB");
        let repository = tokio::task::spawn_blocking(|| {
            Repository::new(config.repository_config)
        })
        .await
        .context(JoinSnafu)?
        .context(RepositorySnafu)?;

        tracing::info!("connected to database; connecting to broker");
        let broker = Broker::new(config.broker_config)
            .await
            .context(BrokerSnafu)?;

        let state = IndexerState::new(&config.dapp_metadata);
        let mut indexer = Indexer {
            repository,
            broker,
            state,
        };

        tracing::info!("connected to broker; starting main loop");
        loop {
            let event = indexer.consume_event().await?;
            let repository = indexer.repository.clone();
            tokio::task::spawn_blocking(move || match event {
                IndexerEvent::Input(input) => {
                    store_input(&repository, input.payload)
                }
                IndexerEvent::Output(output) => {
                    store_output(&repository, output.payload)
                }
            })
            .await
            .context(JoinSnafu)?
            .context(RepositorySnafu)?;
        }
    }

    #[tracing::instrument(level = "trace", skip_all)]
    async fn consume_event(&mut self) -> Result<IndexerEvent, IndexerError> {
        tracing::info!(?self.state, "waiting for next event");
        loop {
            match self.broker.indexer_consume(&mut self.state).await {
                Ok(event) => {
                    tracing::info!(?event, "received event");
                    return Ok(event);
                }
                Err(source) => match source {
                    BrokerError::ConsumeTimeout => {
                        tracing::trace!("broker timed out, trying again");
                        continue;
                    }
                    _ => {
                        return Err(IndexerError::BrokerError { source });
                    }
                },
            }
        }
    }
}

#[tracing::instrument(level = "trace", skip_all)]
fn store_input(
    repository: &Repository,
    input: RollupsInput,
) -> Result<(), rollups_data::Error> {
    match input.data {
        RollupsData::AdvanceStateInput(input) => {
            repository.insert_input(convert_input(input))
        }
        RollupsData::FinishEpoch {} => {
            tracing::trace!("ignoring finish epoch");
            Ok(())
        }
    }
}

#[tracing::instrument(level = "trace", skip_all)]
fn store_output(
    repository: &Repository,
    output: RollupsOutput,
) -> Result<(), rollups_data::Error> {
    match output {
        RollupsOutput::Voucher(voucher) => {
            repository.insert_voucher(convert_voucher(voucher))
        }
        RollupsOutput::Notice(notice) => {
            repository.insert_notice(convert_notice(notice))
        }
        RollupsOutput::Report(report) => {
            repository.insert_report(convert_report(report))
        }
        RollupsOutput::Proof(proof) => {
            repository.insert_proof(convert_proof(proof))
        }
    }
}
