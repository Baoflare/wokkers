use std::{
	convert::{TryFrom, TryInto},
	net::{IpAddr, Ipv4Addr},
};

use chirp_workflow::prelude::*;
use rivet_operation::prelude::proto::backend;
use serde::{Deserialize, Serialize};
use strum::FromRepr;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Cluster {
	pub cluster_id: Uuid,
	pub name_id: String,
	/// Unset for the default cluster.
	pub owner_team_id: Option<Uuid>,
	pub create_ts: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
pub struct Datacenter {
	pub datacenter_id: Uuid,
	pub cluster_id: Uuid,
	pub name_id: String,
	pub display_name: String,
	pub provider: Provider,
	pub provider_datacenter_id: String,
	pub provider_api_token: Option<String>,
	pub pools: Vec<Pool>,
	pub build_delivery_method: BuildDeliveryMethod,
	pub prebakes_enabled: bool,
	pub create_ts: i64,
}

#[derive(Serialize, Deserialize, Hash, Debug, Clone, Copy, PartialEq, Eq, FromRepr)]
pub enum Provider {
	/// Servers are manually provisioned and connected.
	Manual = 1,
	Linode = 0,
}

impl From<rivet_config::config::rivet::Provider> for Provider {
	fn from(value: rivet_config::config::rivet::Provider) -> Provider {
		match value {
			rivet_config::config::rivet::Provider::Linode => Provider::Linode,
		}
	}
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
pub struct Pool {
	pub pool_type: PoolType,
	/// See docs on failover (/docs/packages/cluster/SERVER_PROVISIONING.md#creating-a-new-server)
	pub hardware: Vec<Hardware>,
	pub desired_count: u32,
	pub min_count: u32,
	pub max_count: u32,
	pub drain_timeout: u64,
}

// Backwards compatibility
impl TryFrom<backend::cluster::Pool> for Pool {
	type Error = GlobalError;

	fn try_from(value: backend::cluster::Pool) -> GlobalResult<Self> {
		Ok(Pool {
			pool_type: unwrap!(PoolType::from_repr(value.pool_type.try_into()?)),
			hardware: value
				.hardware
				.iter()
				.map(|h| Hardware {
					provider_hardware: h.provider_hardware.clone(),
				})
				.collect(),
			desired_count: value.desired_count,
			min_count: value.min_count,
			max_count: value.max_count,
			drain_timeout: value.drain_timeout,
		})
	}
}

#[derive(Serialize, Deserialize, Hash, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, FromRepr)]
pub enum PoolType {
	Job = 0,
	Gg = 1,
	Ats = 2,
	Pegboard = 3,
	PegboardIsolate = 4,
}

impl std::fmt::Display for PoolType {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			PoolType::Job => write!(f, "job"),
			PoolType::Gg => write!(f, "gg"),
			PoolType::Ats => write!(f, "ats"),
			PoolType::Pegboard => write!(f, "pegboard"),
			PoolType::PegboardIsolate => write!(f, "pegboard-isolate"),
		}
	}
}

impl From<rivet_config::config::rivet::PoolType> for PoolType {
	fn from(value: rivet_config::config::rivet::PoolType) -> PoolType {
		match value {
			rivet_config::config::rivet::PoolType::Job => PoolType::Job,
			rivet_config::config::rivet::PoolType::Gg => PoolType::Gg,
			rivet_config::config::rivet::PoolType::Ats => PoolType::Ats,
			rivet_config::config::rivet::PoolType::Pegboard => PoolType::Pegboard,
			rivet_config::config::rivet::PoolType::PegboardIsolate => PoolType::PegboardIsolate,
		}
	}
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash)]
pub struct Hardware {
	pub provider_hardware: String,
}

impl From<rivet_config::config::rivet::Hardware> for Hardware {
	fn from(value: rivet_config::config::rivet::Hardware) -> Hardware {
		Hardware {
			provider_hardware: value.name,
		}
	}
}

#[derive(Debug, Serialize, Deserialize, Hash)]
pub struct PoolUpdate {
	pub pool_type: PoolType,

	// Each can be optionally updated
	pub hardware: Vec<Hardware>,
	pub desired_count: Option<u32>,
	pub min_count: Option<u32>,
	pub max_count: Option<u32>,
	pub drain_timeout: Option<u64>,
}

#[derive(Serialize, Deserialize, Hash, Debug, Clone, Copy, PartialEq, Eq, FromRepr)]
pub enum BuildDeliveryMethod {
	TrafficServer = 0,
	S3Direct = 1,
}

impl From<rivet_config::config::rivet::BuildDeliveryMethod> for BuildDeliveryMethod {
	fn from(value: rivet_config::config::rivet::BuildDeliveryMethod) -> BuildDeliveryMethod {
		match value {
			rivet_config::config::rivet::BuildDeliveryMethod::TrafficServer => {
				BuildDeliveryMethod::TrafficServer
			}
			rivet_config::config::rivet::BuildDeliveryMethod::S3Direct => {
				BuildDeliveryMethod::S3Direct
			}
		}
	}
}

#[derive(Debug)]
pub struct Server {
	pub server_id: Uuid,
	pub datacenter_id: Uuid,
	pub pool_type: PoolType,
	pub provider_server_id: Option<String>,
	pub vlan_ip: Option<IpAddr>,
	pub public_ip: Option<IpAddr>,
	pub cloud_destroy_ts: Option<i64>,
}

#[derive(Debug, Default, Clone)]
pub struct Filter {
	pub server_ids: Option<Vec<Uuid>>,
	pub datacenter_ids: Option<Vec<Uuid>>,
	pub cluster_ids: Option<Vec<Uuid>>,
	pub pool_types: Option<Vec<PoolType>>,
	pub public_ips: Option<Vec<Ipv4Addr>>,
}

#[derive(Serialize, Deserialize, Hash, Debug, Clone, Copy, PartialEq, Eq, FromRepr)]
pub enum TlsState {
	Creating = 0,
	Active = 1,
	Renewing = 2,
}
