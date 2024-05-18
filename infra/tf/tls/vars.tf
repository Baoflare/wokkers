variable "namespace" {
	type = string
}

# MARK: DNS
variable "domain_main" {
	type = string
}

variable "domain_cdn" {
	type = string
}

variable "domain_job" {
	type = string
}

# MARK: Nomad
variable "edge_enabled" {
	type = bool
}

# MARK: Prometheus
variable "prometheus_enabled" {
	type = bool
}

# MARK: OpenGB
variable "opengb_enabled" {
	type = bool
}

# MARK: K8s
variable "kubeconfig_path" {
	type = string
}

# MARK: S3
variable "s3_providers" {
	type = map(object({
		endpoint_internal = string
		endpoint_external = string
		region = string
	}))
}

# MARK: Imagor
variable "imagor_enabled" {
	type = bool
}
