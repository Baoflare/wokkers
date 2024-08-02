use chirp_workflow::prelude::*;
use serde_json::json;

pub struct Setup {
	pub server_id: Uuid,
	pub datacenter_id: Uuid,
	pub cluster_id: Uuid,
	pub pool_type: cluster::types::PoolType,
	pub drain_timeout: u64,
}

pub struct SetupRes {
	pub pools: Vec<cluster::types::Pool>,
	pub provider: cluster::types::Provider,
}

pub async fn setup(ctx: &TestCtx, opts: Setup) -> SetupRes {
	let pools = vec![cluster::types::Pool {
		pool_type: opts.pool_type.clone(),
		hardware: vec![cluster::types::Hardware {
			provider_hardware: cluster::util::test::LINODE_HARDWARE.to_string(),
		}],
		desired_count: 0,
		min_count: 0,
		max_count: 0,
		drain_timeout: opts.drain_timeout,
	}];
	let provider = cluster::types::Provider::Linode;

	let mut sub = ctx
		.subscribe::<cluster::workflows::cluster::CreateComplete>(&json!({
			"cluster_id": opts.cluster_id,
		}))
		.await
		.unwrap();

	ctx.dispatch_tagged_workflow(
		&json!({
			"cluster_id": opts.cluster_id,
		}),
		cluster::workflows::cluster::Input {
			cluster_id: opts.cluster_id,
			name_id: util::faker::ident(),
			owner_team_id: None,
		},
	)
	.await
	.unwrap();

	sub.next().await.unwrap();

	let mut sub = ctx
		.subscribe::<cluster::workflows::datacenter::CreateComplete>(&json!({
			"datacenter_id": opts.datacenter_id,
		}))
		.await
		.unwrap();

	ctx.tagged_signal(
		&json!({
			"cluster_id": opts.cluster_id,
		}),
		cluster::workflows::cluster::DatacenterCreate {
			datacenter_id: opts.datacenter_id,
			name_id: util::faker::ident(),
			display_name: util::faker::ident(),

			provider: provider.clone(),
			provider_datacenter_id: "us-southeast".to_string(),
			provider_api_token: None,

			pools: pools.clone(),

			build_delivery_method: cluster::types::BuildDeliveryMethod::TrafficServer,
			prebakes_enabled: false,
		},
	)
	.await
	.unwrap();

	sub.next().await.unwrap();

	// Write new server to db
	sql_execute!(
		[ctx]
		"
		INSERT INTO db_cluster.servers (
			server_id,
			datacenter_id,
			pool_type2,
			create_ts,

			-- Backwards compatibility
			pool_type
		)
		VALUES ($1, $2, $3, $4, 0)
		",
		opts.server_id,
		opts.datacenter_id,
		serde_json::to_string(&opts.pool_type)?,
		util::timestamp::now(),
	)
	.await
	.unwrap();

	SetupRes { pools, provider }
}
