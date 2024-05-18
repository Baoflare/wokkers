use std::collections::HashMap;

use anyhow::{Context, Result};
use indoc::{formatdoc, indoc};
use serde_json::json;
use tokio::fs;

use crate::{
	config::{
		ns,
		service::{RuntimeKind, UploadPolicy},
	},
	context::ProjectContext,
	dep,
	utils::media_resize,
};

pub async fn project(ctx: &ProjectContext) {
	// Init all Terraform projects in parallel
	let init_futs = crate::tasks::infra::all_terraform_plans(ctx)
		.unwrap()
		.into_iter()
		.map(|plan_id| {
			let ctx = ctx.clone();
			async move {
				// Generate helper resources
				gen_bolt_tf(&ctx, &plan_id).await?;

				// Initiate Terraform
				super::cli::init_if_needed(&ctx, &plan_id).await;

				Result::<()>::Ok(())
			}
		});
	futures_util::future::try_join_all(init_futs).await.unwrap();

	vars(ctx).await;
}

/// Generates the `_bolt.tf` file that lives inside the Terraform project to
/// help configure backends and remote states.
pub async fn gen_bolt_tf(ctx: &ProjectContext, plan_id: &str) -> Result<()> {
	// Configure the backend
	let backend = match ctx.ns().terraform.backend {
		ns::TerraformBackend::Local {} => String::new(),
		ns::TerraformBackend::Postgres {} => indoc!(
			"
			terraform {
				backend \"pg\" {}
			}
			"
		)
		.to_string(),
	};

	// Generate the remote states
	let remote_states =
		if let Some(remote_states) = super::remote_states::dependency_graph(ctx).get(plan_id) {
			let variables = match ctx.ns().terraform.backend {
				ns::TerraformBackend::Local {} => String::new(),
				ns::TerraformBackend::Postgres {} => indoc!(
					"
					variable \"remote_state_pg_conn_str\" {
						type = string
					}
					"
				)
				.to_string(),
			};

			let blocks = remote_states
				.iter()
				.map(|remote| gen_remote_state(ctx, remote))
				.collect::<Vec<_>>()
				.join("\n\n");

			format!("{variables}\n\n{blocks}")
		} else {
			String::new()
		};

	let bolt_tf =
		format!("# This is generated by Bolt. Do not modify.\n\n{backend}\n\n{remote_states}");
	let path = ctx.tf_path().join(plan_id).join("_bolt.tf");

	tokio::fs::write(&path, bolt_tf)
		.await
		.context(format!("write _bolt.tf to {}", path.display()))?;

	Ok(())
}

/// Generates a `terraform_remote_state` block for Terraform.
fn gen_remote_state(
	ctx: &ProjectContext,
	remote_state: &super::remote_states::RemoteState,
) -> String {
	let plan_id = &remote_state.plan_id;
	let data_name = remote_state.data_name();
	let workspace =
		dep::terraform::cli::build_localized_workspace_name(ctx.ns_id(), remote_state.plan_id);

	let meta = if let Some(condition) = &remote_state.condition {
		format!("count = {condition} ? 1 : 0")
	} else {
		String::new()
	};

	match ctx.ns().terraform.backend {
		ns::TerraformBackend::Local {} => formatdoc!(
			"
			data \"terraform_remote_state\" \"{data_name}\" {{
				{meta}

				backend = \"local\"

				config = {{
					path = \"../{plan_id}/terraform.tfstate.d/{workspace}/terraform.tfstate\"
				}}
			}}
			"
		),
		ns::TerraformBackend::Postgres {} => formatdoc!(
			"
			data \"terraform_remote_state\" \"{data_name}\" {{
				{meta}

				backend = \"pg\"
				workspace = \"{workspace}\"

				config = {{
					conn_str = var.remote_state_pg_conn_str
				}}
			}}
			"
		),
	}
}

async fn vars(ctx: &ProjectContext) {
	let all_svc = ctx.all_services().await;
	let config = ctx.ns();
	let ns = ctx.ns_id();

	let mut vars = HashMap::<String, serde_json::Value>::new();

	// Namespace
	vars.insert("namespace".into(), json!(ns));

	vars.insert("minio_port".into(), json!(null));

	match &config.cluster.kind {
		ns::ClusterKind::SingleNode {
			public_ip,
			api_http_port,
			api_https_port,
			minio_port,
			tunnel_port,
			..
		} => {
			vars.insert("deploy_method_local".into(), json!(true));
			vars.insert("deploy_method_cluster".into(), json!(false));
			vars.insert("public_ip".into(), json!(public_ip));
			vars.insert("api_http_port".into(), json!(api_http_port));
			vars.insert("api_https_port".into(), json!(api_https_port));
			vars.insert("tunnel_port".into(), json!(tunnel_port));

			// Expose Minio on a dedicated port if DNS not enabled
			if config.dns.is_none() && config.s3.providers.minio.is_some() {
				vars.insert("minio_port".into(), json!(minio_port));
			} else {
				vars.insert("minio_port".into(), json!(null));
			}
		}
		ns::ClusterKind::Distributed {} => {
			vars.insert("deploy_method_local".into(), json!(false));
			vars.insert("deploy_method_cluster".into(), json!(true));
		}
	}

	// Remote state
	match ctx.ns().terraform.backend {
		ns::TerraformBackend::Local {} => {}
		ns::TerraformBackend::Postgres {} => {
			let remote_state_pg_conn_str = ctx
				.read_secret(&["terraform", "pg_backend", "conn_str"])
				.await
				.unwrap();
			vars.insert(
				"remote_state_pg_conn_str".into(),
				json!(remote_state_pg_conn_str),
			);
		}
	}

	// Project
	vars.insert(
		"project_root".into(),
		json!(ctx.path().display().to_string()),
	);

	// Domains
	vars.insert("domain_main".into(), json!(ctx.domain_main()));
	vars.insert("domain_cdn".into(), json!(ctx.domain_cdn()));
	vars.insert("domain_job".into(), json!(ctx.domain_job()));
	vars.insert("domain_main_api".into(), json!(ctx.domain_main_api()));
	vars.insert(
		"dns_deprecated_subdomains".into(),
		json!(config
			.dns
			.as_ref()
			.map_or(false, |x| x.deprecated_subdomains)),
	);
	vars.insert("tls_enabled".into(), json!(ctx.tls_enabled()));

	// Cloudflare
	if let Some(dns) = &config.dns {
		match &dns.provider {
			Some(ns::DnsProvider::Cloudflare { account_id, .. }) => {
				vars.insert("cloudflare_account_id".into(), json!(account_id));
			}
			None => {}
		}
	}

	// Edge nodes
	vars.insert(
		"edge_enabled".into(),
		json!(config.rivet.provisioning.is_some()),
	);

	// Tunnels
	if let Some(ns::Dns {
		provider: Some(ns::DnsProvider::Cloudflare { access, .. }),
		..
	}) = &config.dns
	{
		let mut tunnels = HashMap::new();

		// Grafana tunnel
		tunnels.insert(
			"grafana",
			json!({
				"name": "Grafana",
				"service": "http://grafana.grafana.svc.cluster.local:80",
				"access_groups": access.as_ref().map(|x| vec![x.groups.engineering.clone()]).unwrap_or_default(),
				"service_tokens": access.as_ref().map(|x| vec![x.services.grafana.clone()]).unwrap_or_default(),
			}),
		);

		// Nomad tunnel
		tunnels.insert(
			"nomad",
			json!({
				"name": "Nomad",
				"service": "http://nomad-server.nomad.svc.cluster.local:4646",
				"access_groups": access.as_ref().map(|x| vec![x.groups.engineering.clone()]).unwrap_or_default(),
				"service_tokens": [],
			}),
		);

		vars.insert("tunnels".into(), json!(&tunnels));
	}

	// Rust
	vars.insert(
		"cargo_target_dir".into(),
		json!(ctx.cargo_target_dir().display().to_string()),
	);

	// Services
	{
		let mut services = HashMap::new();
		for (service_id, service) in &config.services {
			services.insert(
				service_id.clone(),
				json!({
					"count": service.count,
					"resources": {
						"cpu": service.resources.cpu,
						"memory": service.resources.memory,
					}
				}),
			);
		}
		vars.insert("services".into(), json!(services));
	}

	// Docker
	vars.insert(
		"authenticate_all_docker_hub_pulls".into(),
		json!(config.docker.authenticate_all_docker_hub_pulls),
	);

	// Extra DNS
	if let Some(dns) = &config.dns {
		let mut extra_dns = Vec::new();
		let domain_main = ctx.domain_main().unwrap();
		let domain_main_api = ctx.domain_main_api().unwrap();

		// Default API domain
		extra_dns.push(json!({
			"zone_name": "main",
			"name": domain_main_api,
		}));

		// OGS
		extra_dns.push(json!({
			"zone_name": "main",
			"name": format!("*.ogs.{domain_main}"),
		}));

		// Add services
		for svc_ctx in all_svc {
			if let Some(router) = svc_ctx.config().kind.router() {
				for mount in &router.mounts {
					if mount.deprecated && !dns.deprecated_subdomains {
						continue;
					}

					if let Some(subdomain) = &mount.subdomain {
						extra_dns.push(json!({
							"zone_name": "main",
							"name": format!("{subdomain}.{domain_main}"),
						}));
					}
				}
			}
		}

		// Add Minio
		let s3_providers = &config.s3.providers;
		if s3_providers.minio.is_some() {
			extra_dns.push(json!({
				"zone_name": "main",
				"name": format!("minio.{}", domain_main),
			}));
		}

		vars.insert("extra_dns".into(), json!(extra_dns));
	}

	// CockroachDB
	match config.cockroachdb.provider {
		ns::CockroachDBProvider::Kubernetes { .. } => {
			vars.insert("cockroachdb_provider".into(), json!("kubernetes"));
		}
		ns::CockroachDBProvider::Managed {
			spend_limit,
			request_unit_limit,
			storage_limit,
		} => {
			vars.insert("cockroachdb_provider".into(), json!("managed"));
			vars.insert("cockroachdb_spend_limit".into(), json!(spend_limit));
			vars.insert(
				"cockroachdb_request_unit_limit".into(),
				json!(request_unit_limit),
			);
			vars.insert("cockroachdb_storage_limit".into(), json!(storage_limit));
		}
	}

	// ClickHouse
	vars.insert(
		"clickhouse_enabled".into(),
		json!(config.clickhouse.is_some()),
	);
	if let Some(clickhouse) = &config.clickhouse {
		match &clickhouse.provider {
			ns::ClickHouseProvider::Kubernetes {} => {
				vars.insert("clickhouse_provider".into(), json!("kubernetes"));
			}
			ns::ClickHouseProvider::Managed { tier } => {
				vars.insert("clickhouse_provider".into(), json!("managed"));
				match tier {
					ns::ClickHouseManagedTier::Development {} => {
						vars.insert("clickhouse_tier".into(), json!("development"));
					}
					ns::ClickHouseManagedTier::Production {
						min_total_memory_gb,
						max_total_memory_gb,
					} => {
						vars.insert("clickhouse_tier".into(), json!("production"));
						vars.insert(
							"clickhouse_min_total_memory_gb".into(),
							json!(min_total_memory_gb),
						);
						vars.insert(
							"clickhouse_max_total_memory_gb".into(),
							json!(max_total_memory_gb),
						);
					}
				}
			}
		}
	} else {
		vars.insert("clickhouse_provider".into(), json!(null));
	}

	if dep::terraform::cli::has_applied(ctx, "clickhouse_k8s").await
		|| dep::terraform::cli::has_applied(ctx, "clickhouse_managed").await
	{
		let clickhouse = dep::terraform::output::read_clickhouse(ctx).await;
		vars.insert("clickhouse_host".into(), json!(*clickhouse.host));
		vars.insert(
			"clickhouse_port_https".into(),
			json!(*clickhouse.port_https),
		);
		vars.insert(
			"clickhouse_port_native_secure".into(),
			json!(*clickhouse.port_native_secure),
		);
	}

	// Prometheus
	vars.insert(
		"prometheus_enabled".into(),
		json!(config.prometheus.is_some()),
	);

	// Redis services
	{
		let mut redis_dbs = HashMap::new();

		// Generate persistent and ephemeral databases
		for svc_ctx in all_svc {
			if let RuntimeKind::Redis { persistent } = svc_ctx.config().runtime {
				if persistent {
					redis_dbs.insert(
						"persistent",
						json!({
							"persistent": true,
						}),
					);
				} else {
					redis_dbs.insert(
						"ephemeral",
						json!({
							"persistent": false,
						}),
					);
				}

				if redis_dbs.len() == 2 {
					break;
				}
			}
		}

		let redis_provider = match &config.redis.provider {
			ns::RedisProvider::Kubernetes { .. } => "kubernetes",
			ns::RedisProvider::Aws { .. } => "aws",
			ns::RedisProvider::Aiven {
				project,
				cloud,
				plan_ephemeral,
				plan_persistent,
			} => {
				vars.insert(
					"redis_aiven".into(),
					json!({
						"project": project,
						"cloud": cloud,
						"plan_ephemeral": plan_ephemeral,
						"plan_persistent": plan_persistent,
					}),
				);

				"aiven"
			}
		};

		vars.insert("redis_replicas".into(), json!(config.redis.replicas));
		vars.insert("redis_provider".into(), json!(redis_provider));
		vars.insert("redis_dbs".into(), json!(redis_dbs));
	}

	// S3
	{
		// Allow testing domains for non-production environments
		let cors_allowed_origins = ctx.s3_cors_allowed_origins();

		let mut s3_buckets = HashMap::<String, serde_json::Value>::new();

		for svc_ctx in all_svc {
			if let RuntimeKind::S3 { upload_policy } = &svc_ctx.config().runtime {
				s3_buckets.insert(
					svc_ctx.s3_bucket_name().await,
					json!({
						"cors_allowed_origins": cors_allowed_origins,
						"policy": match upload_policy {
							UploadPolicy::None => "none",
							UploadPolicy::Download => "download",
							UploadPolicy::Public => "public",
							UploadPolicy::Upload => "upload",
						},
					}),
				);
			}
		}

		vars.insert("s3_buckets".into(), json!(s3_buckets));

		vars.insert(
			"s3_default_provider".into(),
			json!(ctx.default_s3_provider().unwrap().0.as_str()),
		);
		vars.insert("s3_providers".into(), s3_providers(ctx).await.unwrap());
	}

	// Better Uptime
	if let Some(better_uptime) = &config.better_uptime {
		// Make sure DNS is enabled
		if config.dns.is_none() {
			panic!("Better Uptime requires DNS to be enabled, since it uses subdomains to monitor services");
		}

		// Make sure there is at least one pool
		let Some(cluster) = config
			.rivet
			.provisioning
			.as_ref()
			.and_then(|p| p.cluster.as_ref())
		else {
			panic!("Better Uptime requires cluster provisioning");
		};

		if cluster.datacenters.is_empty() {
			panic!("Better Uptime requires at least one datacenter, otherwise it will not be able to monitor");
		};

		// Create monitors
		let mm_monitors = cluster
			.datacenters
			.iter()
			.map(|(name_id, dc)| {
				json!({
					"id": dc.datacenter_id,
					"url": format!("{}/status/matchmaker?region={}", ctx.origin_api(), name_id),
					"public_name": dc.display_name,
				})
			})
			.collect::<Vec<_>>();

		vars.insert(
			"better_uptime_groups".into(),
			json!([
				{
					"id": "mm",
					"name": "Matchmaker",
					"monitors": mm_monitors,
				},
				{
					"id": "cdn",
					"name": "CDN",
					"monitors": [
						{
							"id": "sandbox",
							"url": format!("https://sandbox.{}", ctx.domain_cdn().unwrap()),
							"public_name": "CDN"
						}
					]
				},
			]),
		);

		vars.insert("better_uptime".into(), json!(better_uptime.to_owned()));
	}

	// Imagor
	vars.insert(
		"imagor_enabled".into(),
		json!(config.rivet.cdn.image_resizing.is_some()),
	);

	// NSFW API
	vars.insert(
		"nsfw_api_enabled".into(),
		json!(config.rivet.upload.nsfw_check.is_some()),
	);

	// Media presets
	if config.rivet.cdn.image_resizing.is_some() {
		vars.insert(
			"imagor_presets".into(),
			json!(media_resize::build_presets(ctx.ns_id())
				.into_iter()
				.map(media_resize::ResizePresetSerialize::from)
				.collect::<Vec<_>>()),
		);
		vars.insert(
			"imagor_cors_allowed_origins".into(),
			json!(ctx.imagor_cors_allowed_origins()),
		);
	}

	vars.insert("kubeconfig_path".into(), json!(ctx.gen_kubeconfig_path()));
	vars.insert(
		"k8s_storage_class".into(),
		json!(match config.kubernetes.provider {
			ns::KubernetesProvider::K3d { .. } => "local-path",
			ns::KubernetesProvider::AwsEks { .. } => "ebs-sc",
		}),
	);
	vars.insert("limit_resources".into(), json!(ctx.limit_resources()));
	vars.insert(
		"k8s_dashboard_enabled".into(),
		json!(config.kubernetes.dashboard_enabled),
	);

	vars.insert(
		"cdn_cache_size_gb".into(),
		json!(config.rivet.cdn.cache_size_gb),
	);

	let tf_gen_path = ctx.gen_tf_env_path();
	let _ = fs::create_dir_all(&tf_gen_path.parent().unwrap()).await;
	let vars_json = serde_json::to_string(&vars).unwrap();
	fs::write(&tf_gen_path, vars_json).await.unwrap();
}

async fn s3_providers(ctx: &ProjectContext) -> Result<serde_json::Value> {
	let mut res = serde_json::Map::with_capacity(1);

	let providers = &ctx.ns().s3.providers;
	if providers.minio.is_some() {
		let s3_config = ctx.s3_config(s3_util::Provider::Minio).await?;
		res.insert(
			"minio".to_string(),
			json!({
				"endpoint_internal": s3_config.endpoint_internal,
				"endpoint_external": s3_config.endpoint_external,
				"region": s3_config.region,
			}),
		);
	}
	if providers.backblaze.is_some() {
		let s3_config = ctx.s3_config(s3_util::Provider::Backblaze).await?;
		res.insert(
			"backblaze".to_string(),
			json!({
				"endpoint_internal": s3_config.endpoint_internal,
				"endpoint_external": s3_config.endpoint_external,
				"region": s3_config.region,
			}),
		);
	}
	if providers.aws.is_some() {
		let s3_config = ctx.s3_config(s3_util::Provider::Aws).await?;
		res.insert(
			"aws".to_string(),
			json!({
				"endpoint_internal": s3_config.endpoint_internal,
				"endpoint_external": s3_config.endpoint_external,
				"region": s3_config.region,
			}),
		);
	}

	Ok(res.into())
}
