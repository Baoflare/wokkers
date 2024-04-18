use proto::backend::pkg::*;
use rivet_operation::prelude::*;
use util_linode::api;

#[derive(sqlx::FromRow)]
struct LinodeData {
	ssh_key_id: i64,
	linode_id: Option<i64>,
	firewall_id: Option<i64>,
}

#[operation(name = "linode-server-destroy")]
pub async fn handle(
	ctx: OperationContext<linode::server_destroy::Request>,
) -> GlobalResult<linode::server_destroy::Response> {
	let server_id = unwrap_ref!(ctx.server_id).as_uuid();
	let datacenter_id = unwrap!(ctx.datacenter_id);

	let datacenter_res = op!([ctx] cluster_datacenter_get {
		datacenter_ids: vec![datacenter_id],
	})
	.await?;
	let datacenter = unwrap!(datacenter_res.datacenters.first());

	let data = sql_fetch_optional!(
		[ctx, LinodeData]
		"
		SELECT ssh_key_id, linode_id, firewall_id
		FROM db_cluster.servers_linode
		WHERE
			server_id = $1 AND
			destroy_ts IS NULL
		",
		server_id,
	)
	.await?;

	let Some(data) = data else {
		tracing::warn!("deleting server that doesn't exist");
		return Ok(linode::server_destroy::Response {});
	};

	// Build HTTP client
	let client = util_linode::Client::new(datacenter.provider_api_token.clone()).await?;

	if let Some(linode_id) = data.linode_id {
		api::delete_instance(&client, linode_id).await?;
	}

	api::delete_ssh_key(&client, data.ssh_key_id).await?;

	if let Some(firewall_id) = data.firewall_id {
		api::delete_firewall(&client, firewall_id).await?;
	}

	// Remove record
	sql_execute!(
		[ctx]
		"
		UPDATE db_cluster.servers_linode
		SET destroy_ts = $2
		WHERE
			server_id = $1 AND
			destroy_ts IS NULL
		",
		server_id,
	)
	.await?;

	Ok(linode::server_destroy::Response {})
}
