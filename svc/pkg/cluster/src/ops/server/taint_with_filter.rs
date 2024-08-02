use std::collections::HashSet;

use chirp_workflow::prelude::*;
use serde_json::json;

use crate::types::Filter;

#[derive(Debug)]
pub struct Input {
	pub filter: Filter,
}

#[derive(Debug)]
pub struct Output {}

#[operation]
pub async fn cluster_server_taint_with_filter(
	ctx: &OperationCtx,
	input: &Input,
) -> GlobalResult<Output> {
	let servers_res = ctx
		.op(crate::ops::server::list::Input {
			filter: input.filter.clone(),
			include_destroyed: false,
		})
		.await?;

	// Flag as tainted
	let server_ids = servers_res
		.servers
		.iter()
		.map(|x| x.server_id)
		.collect::<Vec<_>>();
	sql_execute!(
		[ctx]
		"
		UPDATE db_cluster.servers
		SET taint_ts = $2
		WHERE server_id = ANY($1)
		",
		&server_ids,
		util::timestamp::now(),
	)
	.await?;

	// Taint servers
	for server_id in server_ids {
		ctx.tagged_signal(
			&json!({
				"server_id": server_id,
			}),
			crate::workflows::server::Taint {},
		)
		.await?;
	}

	// Trigger scale event
	let dc_ids = servers_res
		.servers
		.iter()
		.map(|x| x.datacenter_id)
		.collect::<HashSet<_>>();
	for dc_id in dc_ids {
		ctx.tagged_signal(
			&json!({
				"datacenter_id": dc_id,
			}),
			crate::workflows::datacenter::Scale {},
		)
		.await?;
	}

	Ok(Output {})
}
