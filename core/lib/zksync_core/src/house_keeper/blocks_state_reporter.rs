use async_trait::async_trait;
use zksync_dal::{ConnectionPool, Core, CoreDal};
use zksync_shared_metrics::{BlockL1Stage, BlockStage, L1StageLatencyLabel, APP_METRICS};
use zksync_utils::time::seconds_since_epoch;

use crate::house_keeper::periodic_job::PeriodicJob;

#[derive(Debug)]
pub struct L1BatchMetricsReporter {
    reporting_interval_ms: u64,
    connection_pool: ConnectionPool<Core>,
}

impl L1BatchMetricsReporter {
    pub fn new(reporting_interval_ms: u64, connection_pool: ConnectionPool<Core>) -> Self {
        Self {
            reporting_interval_ms,
            connection_pool,
        }
    }

    async fn report_metrics(&self) {
        let mut block_metrics = vec![];
        let mut conn = self.connection_pool.connection().await.unwrap();
        let last_l1_batch = conn
            .blocks_dal()
            .get_sealed_l1_batch_number()
            .await
            .unwrap();
        if let Some(number) = last_l1_batch {
            block_metrics.push((number, BlockStage::Sealed));
        }

        let last_l1_batch_with_metadata = conn
            .blocks_dal()
            .get_last_l1_batch_number_with_metadata()
            .await
            .unwrap();
        if let Some(number) = last_l1_batch_with_metadata {
            block_metrics.push((number, BlockStage::MetadataCalculated));
        }

        let eth_stats = conn.eth_sender_dal().get_eth_l1_batches().await.unwrap();
        for (tx_type, l1_batch) in eth_stats.saved {
            let stage = BlockStage::L1 {
                l1_stage: BlockL1Stage::Saved,
                tx_type,
            };
            block_metrics.push((l1_batch, stage))
        }

        for (tx_type, l1_batch) in eth_stats.mined {
            let stage = BlockStage::L1 {
                l1_stage: BlockL1Stage::Mined,
                tx_type,
            };
            block_metrics.push((l1_batch, stage))
        }

        // TODO (PLA-335): Restore prover and witgen metrics

        for (l1_batch_number, stage) in block_metrics {
            APP_METRICS.block_number[&stage].set(l1_batch_number.0.into());
        }

        let oldest_uncommitted_batch_timestamp = conn
            .blocks_dal()
            .oldest_uncommitted_batch_timestamp()
            .await
            .unwrap();
        let oldest_unproved_batch_timestamp = conn
            .blocks_dal()
            .oldest_unproved_batch_timestamp()
            .await
            .unwrap();
        let oldest_unexecuted_batch_timestamp = conn
            .blocks_dal()
            .oldest_unexecuted_batch_timestamp()
            .await
            .unwrap();

        let now = seconds_since_epoch();

        if let Some(timestamp) = oldest_uncommitted_batch_timestamp {
            APP_METRICS.blocks_state_block_eth_stage_latency
                [&L1StageLatencyLabel::UncommittedBlock]
                .set(now.saturating_sub(timestamp));
        }
        if let Some(timestamp) = oldest_unproved_batch_timestamp {
            APP_METRICS.blocks_state_block_eth_stage_latency[&L1StageLatencyLabel::UnprovedBlock]
                .set(now.saturating_sub(timestamp));
        }
        if let Some(timestamp) = oldest_unexecuted_batch_timestamp {
            APP_METRICS.blocks_state_block_eth_stage_latency[&L1StageLatencyLabel::UnexecutedBlock]
                .set(now.saturating_sub(timestamp));
        }
    }
}

#[async_trait]
impl PeriodicJob for L1BatchMetricsReporter {
    const SERVICE_NAME: &'static str = "L1BatchMetricsReporter";

    async fn run_routine_task(&mut self) -> anyhow::Result<()> {
        self.report_metrics().await;
        Ok(())
    }

    fn polling_interval_ms(&self) -> u64 {
        self.reporting_interval_ms
    }
}
