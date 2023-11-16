locals {
	redis_k8s = var.redis_provider == "kubernetes"
	service_redis = lookup(var.services, "redis", {
		count = 3
		resources = {
			cpu = 1000
			memory = 2000
		}
	})

	redis_svcs = local.redis_k8s ? {
		for k, v in var.redis_dbs:
		k => {
			persistent = v.persistent
			password = module.redis_secrets[0].values["redis/${k}/password"]
		}
		if local.redis_k8s
	} : {}

	redis_node_config = {
		for k, v in var.redis_dbs:
		k => {
			priorityClassName = kubernetes_priority_class.redis_priority.metadata.0.name
			resources = var.limit_resources ? {
				limits = {
					memory = "${local.service_redis.resources.memory}Mi"
					cpu = "${local.service_redis.resources.cpu}m"
				}
			} : null
			persistence = {
				enabled = v.persistent
			}
		}
	}
}

module "redis_secrets" {
	count = local.redis_k8s ? 1 : 0

	source = "../modules/secrets"

	keys = [
		for k, v in var.redis_dbs: "redis/${k}/password"
	]
}

resource "kubernetes_namespace" "redis" {
	depends_on = [ helm_release.prometheus ]
	for_each = var.redis_dbs

	metadata {
		name = "redis-${each.key}"
	}
}

resource "kubernetes_priority_class" "redis_priority" {
	metadata {
		name = "redis-priority"
	}
	value = 40
}

resource "helm_release" "redis" {
	depends_on = [helm_release.prometheus]
	for_each = local.redis_svcs

	name = "redis"
	namespace = kubernetes_namespace.redis[each.key].metadata.0.name
	repository = "https://charts.bitnami.com/bitnami"
	chart = "redis"
	version = "18.4.0"
	values = [yamlencode({
		architecture = "replication"

		commonConfiguration = <<EOF
		# Enable AOF https://redis.io/topics/persistence#append-only-file
		appendonly ${each.value.persistent ? "yes" : "no"}

		# Disable RDB persistence. AOF persistence already enabled on persistent,
		# not needed on ephemeral.
		save ""

		# Use allkeys-lru instead of volatile-lru because we don't want the cache nodes to crash if they run out of memory
		maxmemory-policy ${each.value.persistent ? "noeviction" : "allkeys-lru"}
		EOF

		global = {
			storageClass = var.k8s_storage_class
			redis = {
				password = each.value.password
			}
		}

		tls = {
			enabled = true
			authClients = false
			autoGenerated = true
		}

		master = merge(local.redis_node_config[each.key], {
			count = 1
		})
		replica = merge(local.redis_node_config[each.key], {
			replicaCount = 1
		})
		sentinel = {
			enabled = true
		}

		metrics = {
			enabled = true
			serviceMonitor = {
				enabled = true
				namespace = kubernetes_namespace.redis[each.key].metadata.0.name
			}
			extraArgs = each.key == "chirp" ? {
				"check-streams" = "'{topic:*}:topic'"
			} : {}

			# TODO:
			# prometheusRule = {
			# 	enabled = true
			# 	namespace = kubernetes_namespace.redis[each.key].metadata.0.name
			# }
		}
	})]
}

data "kubernetes_secret" "redis_ca" {
	for_each = var.redis_dbs

	depends_on = [helm_release.redis]

	metadata {
		name = "redis-crt"
		namespace = kubernetes_namespace.redis[each.key].metadata.0.name
	}
}

resource "kubernetes_config_map" "redis_ca" {
	for_each = merge([
		for ns in ["rivet-service", "bolt"]: {
			for k, v in var.redis_dbs:
				"${k}-${ns}" => {
				db = k
				namespace = ns
			}
		}
	]...)

	metadata {
		name = "redis-${each.value.db}-ca"
		namespace = each.value.namespace
	}

	data = {
		"ca.crt" = data.kubernetes_secret.redis_ca[each.value.db].data["ca.crt"]
	}
}

