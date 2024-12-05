use anyhow::*;
use deno_core::{serde_v8::GlobalValue, v8};
use pegboard::protocol;
use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JsMetadata {
	pub actor: JsMetadataActor,
	pub env: JsMetadataEnv,
	pub region: JsMetadataRegion,
	pub cluster: JsMetadataCluster,
	pub build: JsMetadataBuild,
}

impl JsMetadata {
	pub fn from_actor(
		actor_id: Uuid,
		metadata: protocol::ActorMetadata,
		scope: &mut v8::HandleScope<'_>,
	) -> Result<Self> {
		let date =
			v8::Local::from(v8::Date::new(scope, metadata.create_ts as f64).context("bad date")?);

		Ok(JsMetadata {
			actor: JsMetadataActor {
				id: actor_id,
				tags: metadata.tags,
				created_at: v8::Global::new(scope, date).into(),
			},
			env: JsMetadataEnv {
				id: metadata.env.env_id,
			},
			region: JsMetadataRegion {
				id: metadata.datacenter.name_id,
				name: metadata.datacenter.display_name,
			},
			cluster: JsMetadataCluster {
				id: metadata.cluster.cluster_id,
			},
			build: JsMetadataBuild {
				id: metadata.build.build_id,
			},
		})
	}
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JsMetadataActor {
	pub id: Uuid,
	pub tags: protocol::HashableMap<String, String>,
	pub created_at: GlobalValue, // v8::Date
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JsMetadataEnv {
	pub id: Uuid,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JsMetadataRegion {
	pub id: String,
	pub name: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JsMetadataCluster {
	pub id: Uuid,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JsMetadataBuild {
	pub id: Uuid,
}
