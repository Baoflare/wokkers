use std::time::Duration;

use chirp_workflow::prelude::*;

fn main() -> GlobalResult<()> {
	rivet_runtime::run(start()).unwrap()
}

async fn start() -> GlobalResult<()> {
	let pools = rivet_pools::from_env("workflow-metrics-publish").await?;

	tokio::task::Builder::new()
		.name("workflow_metrics_publish::health_checks")
		.spawn(rivet_health_checks::run_standalone(
			rivet_health_checks::Config {
				pools: Some(pools.clone()),
			},
		))?;

	tokio::task::Builder::new()
		.name("workflow_metrics_publish::metrics")
		.spawn(rivet_metrics::run_standalone())?;

	let mut interval = tokio::time::interval(Duration::from_secs(5));
	loop {
		interval.tick().await;

		workflow_metrics_publish::run_from_env(pools.clone()).await?;
	}
}
