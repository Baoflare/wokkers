use global_error::GlobalResult;
use tokio::time::Duration;

use crate::{util, DatabaseHandle, RegistryHandle, WorkflowCtx};

const TICK_INTERVAL: Duration = Duration::from_millis(50);

/// Used to spawn a new thread that indefinitely polls the database for new workflows. Only pulls workflows
/// that are registered in its registry. After pulling, the workflows are ran and their state is written to
/// the database.
pub struct Worker {
	registry: RegistryHandle,
	db: DatabaseHandle,
}

impl Worker {
	pub fn new(registry: RegistryHandle, db: DatabaseHandle) -> Self {
		Worker { registry, db }
	}

	pub async fn start(mut self, pools: rivet_pools::Pools) -> GlobalResult<()> {
		let mut interval = tokio::time::interval(TICK_INTERVAL);

		let shared_client = chirp_client::SharedClient::from_env(pools.clone())?;
		let cache = rivet_cache::CacheInner::from_env(pools.clone())?;

		loop {
			interval.tick().await;
			self.tick(&shared_client, &pools, &cache).await?;
		}
	}

	// Query the database for new workflows and run them.
	async fn tick(
		&mut self,
		shared_client: &chirp_client::SharedClientHandle,
		pools: &rivet_pools::Pools,
		cache: &rivet_cache::Cache,
	) -> GlobalResult<()> {
		tracing::trace!("tick");

		let registered_workflows = self
			.registry
			.workflows
			.keys()
			.map(|k| k.as_str())
			.collect::<Vec<_>>();

		// Query awake workflows
		let workflows = self.db.pull_workflows(&registered_workflows).await?;
		for workflow in workflows {
			let client = shared_client.clone().wrap_new(&workflow.workflow_name);
			let conn = rivet_connection::Connection::new(client, pools.clone(), cache.clone());

			let wake_deadline_ts = workflow.wake_deadline_ts;
			let ctx = WorkflowCtx::new(self.registry.clone(), self.db.clone(), conn, workflow)?;

			tokio::task::spawn(async move {
				// Sleep until deadline
				if let Some(wake_deadline_ts) = wake_deadline_ts {
					util::sleep_until_ts(wake_deadline_ts).await;
				}

				ctx.run_workflow().await;
			});
		}

		Ok(())
	}
}
