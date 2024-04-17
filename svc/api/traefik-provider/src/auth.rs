use api_helper::auth::{ApiAuth, AuthRateLimitCtx};
use proto::claims::Claims;
use rivet_operation::prelude::*;

/// Information derived from the authentication middleware.
pub struct Auth {
	_claims: Option<Claims>,
}

#[async_trait]
impl ApiAuth for Auth {
	async fn new(
		_api_token: Option<String>,
		rate_limit_ctx: AuthRateLimitCtx<'_>,
	) -> GlobalResult<Auth> {
		Self::rate_limit(rate_limit_ctx).await?;

		Ok(Auth { _claims: None })
	}

	async fn rate_limit(_rate_limit_ctx: AuthRateLimitCtx<'_>) -> GlobalResult<()> {
		Ok(())
	}
}

impl Auth {
	pub async fn token(&self, token: &str) -> GlobalResult<()> {
		ensure_eq_with!(
			token,
			&util::env::read_secret(&["rivet", "api_traefik_provider", "token"]).await?,
			API_FORBIDDEN,
			reason = "Invalid token",
		);

		Ok(())
	}
}
